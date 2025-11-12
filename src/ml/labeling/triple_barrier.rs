use super::config::{BarrierType, Label, LabeledSignal, LabelingConfig};
use crate::ml::signals::types::*;
use crate::error::TradebiasError;
use polars::prelude::*;

pub struct TripleBarrierLabeler {
    config: LabelingConfig,
}

impl TripleBarrierLabeler {
    pub fn new(config: LabelingConfig) -> Self {
        Self { config }
    }

    /// Label all signals using triple-barrier method
    pub fn label(
        &self,
        signal_dataset: &SignalDataset,
    ) -> Result<Vec<LabeledSignal>, TradebiasError> {
        let close = signal_dataset.market_data.column("close")?.f64()?;
        let high = signal_dataset.market_data.column("high")?.f64()?;
        let low = signal_dataset.market_data.column("low")?.f64()?;

        // Calculate ATR if needed
        let atr_values = if self.config.use_atr_based {
            Some(self.calculate_atr_series(&signal_dataset.market_data)?)
        } else {
            None
        };

        let mut labeled_signals = Vec::new();

        for signal in signal_dataset.signals.iter() {
            let labeled = self.label_single_signal(
                signal,
                close,
                high,
                low,
                atr_values.as_ref(),
            )?;

            labeled_signals.push(labeled);
        }

        Ok(labeled_signals)
    }

    fn label_single_signal(
        &self,
        signal: &Signal,
        close: &Float64Chunked,
        high: &Float64Chunked,
        low: &Float64Chunked,
        atr_values: Option<&Vec<f64>>,
    ) -> Result<LabeledSignal, TradebiasError> {
        let entry_idx = signal.bar_index;
        let entry_price = close.get(entry_idx)
            .ok_or_else(|| TradebiasError::Validation("Invalid entry index".to_string()))?;

        // Calculate barriers
        let (profit_target, stop_loss) = if self.config.use_atr_based {
            let atr = atr_values
                .and_then(|v| v.get(entry_idx).copied())
                .unwrap_or(entry_price * 0.01);

            (
                entry_price + (atr * self.config.atr_profit_multiple),
                entry_price - (atr * self.config.atr_stop_multiple),
            )
        } else {
            (
                entry_price * (1.0 + self.config.profit_target_pct),
                entry_price * (1.0 - self.config.stop_loss_pct),
            )
        };

        let max_idx = (entry_idx + self.config.time_limit_bars).min(close.len());

        // Scan forward to find first barrier hit
        for i in (entry_idx + 1)..max_idx {
            let bar_high = high.get(i).unwrap_or(0.0);
            let bar_low = low.get(i).unwrap_or(0.0);

            // Check profit target (use high for long signals)
            match signal.direction {
                SignalDirection::Long => {
                    if bar_high >= profit_target {
                        let return_pct = (profit_target - entry_price) / entry_price;
                        return Ok(LabeledSignal {
                            signal_idx: entry_idx,
                            label: Label::Profit,
                            bars_held: i - entry_idx,
                            return_pct,
                            hit_barrier: BarrierType::Upper,
                        });
                    }

                    if bar_low <= stop_loss {
                        let return_pct = (stop_loss - entry_price) / entry_price;
                        return Ok(LabeledSignal {
                            signal_idx: entry_idx,
                            label: Label::Loss,
                            bars_held: i - entry_idx,
                            return_pct,
                            hit_barrier: BarrierType::Lower,
                        });
                    }
                }
                SignalDirection::Short => {
                    // Inverse for short signals
                    if bar_low <= profit_target {
                        let return_pct = (entry_price - profit_target) / entry_price;
                        return Ok(LabeledSignal {
                            signal_idx: entry_idx,
                            label: Label::Profit,
                            bars_held: i - entry_idx,
                            return_pct,
                            hit_barrier: BarrierType::Upper,
                        });
                    }

                    if bar_high >= stop_loss {
                        let return_pct = (entry_price - stop_loss) / entry_price;
                        return Ok(LabeledSignal {
                            signal_idx: entry_idx,
                            label: Label::Loss,
                            bars_held: i - entry_idx,
                            return_pct,
                            hit_barrier: BarrierType::Lower,
                        });
                    }
                }
            }
        }

        // Hit time limit
        let exit_price = close.get(max_idx - 1).unwrap_or(entry_price);
        let return_pct = match signal.direction {
            SignalDirection::Long => (exit_price - entry_price) / entry_price,
            SignalDirection::Short => (entry_price - exit_price) / entry_price,
        };

        Ok(LabeledSignal {
            signal_idx: entry_idx,
            label: Label::Timeout,
            bars_held: max_idx - entry_idx,
            return_pct,
            hit_barrier: BarrierType::Vertical,
        })
    }

    fn calculate_atr_series(&self, data: &DataFrame) -> Result<Vec<f64>, TradebiasError> {
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let close = data.column("close")?.f64()?;

        let window = 14;
        let mut atr_values = Vec::new();

        for i in 0..close.len() {
            if i < window {
                atr_values.push(0.0);
                continue;
            }

            let mut tr_sum = 0.0;
            for j in (i - window)..i {
                if let (Some(h), Some(l), Some(prev_c)) = (
                    high.get(j),
                    low.get(j),
                    close.get(j.saturating_sub(1)),
                ) {
                    let tr = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
                    tr_sum += tr;
                }
            }

            atr_values.push(tr_sum / window as f64);
        }

        Ok(atr_values)
    }

    /// Analyze label distribution
    pub fn analyze_distribution(labels: &[LabeledSignal]) -> LabelStats {
        let mut stats = LabelStats::default();

        for labeled in labels {
            match labeled.label {
                Label::Profit => {
                    stats.profit_count += 1;
                    stats.profit_returns.push(labeled.return_pct);
                }
                Label::Loss => {
                    stats.loss_count += 1;
                    stats.loss_returns.push(labeled.return_pct);
                }
                Label::Timeout => {
                    stats.timeout_count += 1;
                    stats.timeout_returns.push(labeled.return_pct);
                }
            }
            stats.total_count += 1;
        }

        stats.profit_pct = (stats.profit_count as f64 / stats.total_count as f64) * 100.0;
        stats.loss_pct = (stats.loss_count as f64 / stats.total_count as f64) * 100.0;
        stats.timeout_pct = (stats.timeout_count as f64 / stats.total_count as f64) * 100.0;

        stats
    }
}

#[derive(Debug, Default)]
pub struct LabelStats {
    pub total_count: usize,
    pub profit_count: usize,
    pub loss_count: usize,
    pub timeout_count: usize,
    pub profit_pct: f64,
    pub loss_pct: f64,
    pub timeout_pct: f64,
    pub profit_returns: Vec<f64>,
    pub loss_returns: Vec<f64>,
    pub timeout_returns: Vec<f64>,
}
