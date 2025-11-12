# 15 - ML Meta-Labeling & Triple-Barrier Method

## Goal
Implement the triple-barrier labeling method and meta-model training for filtering trading signals. This ML layer predicts signal quality to improve strategy performance.

## Prerequisites
- **14-ml-feature-engineering.md** - Feature extraction
- **07-backtesting-engine.md** - Trade simulation
- **08-metrics-engine.md** - Performance metrics

## What You'll Create
1. `TripleBarrierLabeler` - Label signals using profit target, stop loss, and time expiration
2. `MetaModel` - Train ensemble models to predict signal quality
3. Sample weighting and purged cross-validation
4. Model calibration and evaluation
5. Signal filtering based on ML predictions

## Triple-Barrier Method

The triple-barrier method labels each signal by the first barrier it hits:

```
Price
  │
  │    ╔═══════════════════════╗  Upper Barrier (Profit Target)
  │    ║                       ║  → Label: +1 (Profitable)
  ├────╫───────┬───────────────╫────────────────────> Time
  │    ║       │               ║
  │    ║       │ Hit time      ║
  │    ║       │ barrier       ║
  │    ║       ▼               ║
  │    ║   ╔═══════════════════╝  Vertical Barrier (Time Expiration)
  │    ║   ║                      → Label: 0 (Timeout)
  │    ╚═══╝                      Lower Barrier (Stop Loss)
  │                               → Label: -1 (Loss)
Signal Time
```

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│            Signal Dataset (from doc 14)               │
│  • N signals                                          │
│  • Features for each signal                           │
└───────────────────┬──────────────────────────────────┘
                    │
                    ↓
┌──────────────────────────────────────────────────────┐
│         TripleBarrierLabeler                          │
│                                                       │
│  For each signal:                                     │
│    1. Set profit target (e.g., +2% from entry)       │
│    2. Set stop loss (e.g., -1% from entry)           │
│    3. Set time limit (e.g., 10 bars)                 │
│    4. Find first barrier hit                         │
│    5. Assign label: +1 (profit), -1 (loss), 0 (time) │
│                                                       │
│  Output: Labels [1, -1, 1, 0, 1, ...]                │
└───────────────────┬──────────────────────────────────┘
                    │
                    ↓
┌──────────────────────────────────────────────────────┐
│         Feature Matrix + Labels                       │
│                                                       │
│   Features (X)          Labels (y)                    │
│   ┌──────────────┐      ┌────┐                       │
│   │ return_5     │      │ +1 │                       │
│   │ rsi_14       │      │ -1 │                       │
│   │ volatility   │      │ +1 │                       │
│   │ ...          │      │ 0  │                       │
│   └──────────────┘      └────┘                       │
└───────────────────┬──────────────────────────────────┘
                    │
                    ↓
┌──────────────────────────────────────────────────────┐
│         MetaModel Training                            │
│                                                       │
│  Base Models:                                         │
│  ├─ RandomForest                                      │
│  ├─ XGBoost                                           │
│  ├─ LogisticRegression                               │
│  └─ SVM                                               │
│                                                       │
│  Ensemble Method:                                     │
│  └─ Soft Voting (weighted average of probabilities)  │
│                                                       │
│  Output: P(signal is profitable)                     │
└───────────────────┬──────────────────────────────────┘
                    │
                    ↓
┌──────────────────────────────────────────────────────┐
│         Signal Filtering                              │
│                                                       │
│  Prediction > threshold (e.g., 0.6)?                 │
│  ├─ YES → Keep signal                                │
│  └─ NO  → Discard signal                             │
│                                                       │
│  Result: Improved strategy performance               │
└──────────────────────────────────────────────────────┘
```

## Implementation

### Step 1: Labeling Configuration

Create `src/ml/labeling/config.rs`:

```rust
#[derive(Debug, Clone)]
pub struct LabelingConfig {
    pub profit_target_pct: f64,  // e.g., 0.02 = 2% profit target
    pub stop_loss_pct: f64,      // e.g., 0.01 = 1% stop loss
    pub time_limit_bars: usize,  // e.g., 10 bars maximum hold time
    pub use_atr_based: bool,     // Use ATR multiples instead of fixed percentages
    pub atr_profit_multiple: f64, // e.g., 2.0 = 2 * ATR for profit target
    pub atr_stop_multiple: f64,   // e.g., 1.0 = 1 * ATR for stop loss
}

impl Default for LabelingConfig {
    fn default() -> Self {
        Self {
            profit_target_pct: 0.02,
            stop_loss_pct: 0.01,
            time_limit_bars: 10,
            use_atr_based: false,
            atr_profit_multiple: 2.0,
            atr_stop_multiple: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Profit = 1,   // Hit profit target
    Loss = -1,    // Hit stop loss
    Timeout = 0,  // Hit time limit
}

#[derive(Debug, Clone)]
pub struct LabeledSignal {
    pub signal_idx: usize,
    pub label: Label,
    pub bars_held: usize,
    pub return_pct: f64,
    pub hit_barrier: BarrierType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    Upper,    // Profit target
    Lower,    // Stop loss
    Vertical, // Time limit
}
```

### Step 2: Triple-Barrier Labeler

Create `src/ml/labeling/triple_barrier.rs`:

```rust
use super::config::*;
use crate::ml::signals::types::*;
use crate::error::TradeBiasError;
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
    ) -> Result<Vec<LabeledSignal>, TradeBiasError> {
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

        for (idx, signal) in signal_dataset.signals.iter().enumerate() {
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
        atr_values: Option<&[f64]>,
    ) -> Result<LabeledSignal, TradeBiasError> {
        let entry_idx = signal.bar_index;
        let entry_price = close.get(entry_idx)
            .ok_or_else(|| TradeBiasError::Validation("Invalid entry index".to_string()))?;

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

    fn calculate_atr_series(&self, data: &DataFrame) -> Result<Vec<f64>, TradeBiasError> {
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
```

### Step 3: Meta-Model Wrapper (Concept)

Create `src/ml/models/meta_model.rs`:

```rust
use crate::error::TradeBiasError;
use polars::prelude::*;

/// Meta-model for predicting signal quality
/// NOTE: This is a conceptual wrapper. Actual ML implementation
/// requires external crates (smartcore, linfa, or Python bridge)
pub struct MetaModel {
    model_type: ModelType,
    trained: bool,
}

#[derive(Debug, Clone)]
pub enum ModelType {
    RandomForest,
    LogisticRegression,
    Ensemble,
}

impl MetaModel {
    pub fn new(model_type: ModelType) -> Self {
        Self {
            model_type,
            trained: false,
        }
    }

    /// Train model on features and labels
    pub fn train(
        &mut self,
        features: &DataFrame,
        labels: &[i32],
    ) -> Result<TrainingMetrics, TradeBiasError> {
        // Placeholder for actual training logic
        // In practice, this would:
        // 1. Convert DataFrame to feature matrix
        // 2. Split into train/validation
        // 3. Train model using ML library
        // 4. Evaluate on validation set
        // 5. Return metrics

        self.trained = true;

        Ok(TrainingMetrics {
            accuracy: 0.65,
            precision: 0.70,
            recall: 0.60,
            f1_score: 0.64,
            roc_auc: 0.72,
        })
    }

    /// Predict probability that signal will be profitable
    pub fn predict_proba(
        &self,
        features: &DataFrame,
    ) -> Result<Vec<f64>, TradeBiasError> {
        if !self.trained {
            return Err(TradeBiasError::Validation(
                "Model not trained yet".to_string(),
            ));
        }

        // Placeholder: return dummy probabilities
        // In practice, this would use the trained model
        let n_samples = features.height();
        Ok(vec![0.6; n_samples])
    }
}

#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub roc_auc: f64,
}
```

### Step 4: Signal Filtering

Create `src/ml/filtering/filter.rs`:

```rust
use crate::ml::models::meta_model::MetaModel;
use crate::ml::signals::types::*;
use crate::error::TradeBiasError;
use polars::prelude::*;

pub struct SignalFilter {
    model: MetaModel,
    threshold: f64,
}

impl SignalFilter {
    pub fn new(model: MetaModel, threshold: f64) -> Self {
        Self { model, threshold }
    }

    /// Filter signals based on ML predictions
    pub fn filter(
        &self,
        signals: &[Signal],
        features: &DataFrame,
    ) -> Result<Vec<Signal>, TradeBiasError> {
        // Get predictions
        let probabilities = self.model.predict_proba(features)?;

        // Filter signals above threshold
        let mut filtered_signals = Vec::new();

        for (i, (signal, &prob)) in signals.iter().zip(probabilities.iter()).enumerate() {
            if prob >= self.threshold {
                filtered_signals.push(signal.clone());
            }
        }

        Ok(filtered_signals)
    }

    /// Analyze filtering impact
    pub fn analyze_impact(
        &self,
        original_signals: &[Signal],
        filtered_signals: &[Signal],
    ) -> FilteringStats {
        FilteringStats {
            original_count: original_signals.len(),
            filtered_count: filtered_signals.len(),
            retention_rate: (filtered_signals.len() as f64 / original_signals.len() as f64) * 100.0,
            rejected_count: original_signals.len() - filtered_signals.len(),
        }
    }
}

#[derive(Debug)]
pub struct FilteringStats {
    pub original_count: usize,
    pub filtered_count: usize,
    pub retention_rate: f64,
    pub rejected_count: usize,
}
```

## Usage Example

```rust
use tradebias::ml::labeling::{TripleBarrierLabeler, LabelingConfig};
use tradebias::ml::models::meta_model::{MetaModel, ModelType};
use tradebias::ml::filtering::filter::SignalFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Extract signals and features (from doc 14)
    let signal_dataset = extract_signals(&ast, &data)?;
    let features = engineer_features(&signal_dataset)?;

    // 2. Label signals using triple-barrier method
    let labeling_config = LabelingConfig {
        profit_target_pct: 0.02,  // 2% profit target
        stop_loss_pct: 0.01,      // 1% stop loss
        time_limit_bars: 10,      // 10 bars max hold
        ..Default::default()
    };

    let labeler = TripleBarrierLabeler::new(labeling_config);
    let labeled_signals = labeler.label(&signal_dataset)?;

    // Analyze distribution
    let stats = TripleBarrierLabeler::analyze_distribution(&labeled_signals);
    println!("Label distribution:");
    println!("  Profit: {:.1}%", stats.profit_pct);
    println!("  Loss: {:.1}%", stats.loss_pct);
    println!("  Timeout: {:.1}%", stats.timeout_pct);

    // 3. Train meta-model
    let labels: Vec<i32> = labeled_signals
        .iter()
        .map(|l| l.label as i32)
        .collect();

    let mut model = MetaModel::new(ModelType::Ensemble);
    let metrics = model.train(&features, &labels)?;

    println!("\nTraining metrics:");
    println!("  Accuracy: {:.2}", metrics.accuracy);
    println!("  Precision: {:.2}", metrics.precision);
    println!("  ROC-AUC: {:.2}", metrics.roc_auc);

    // 4. Filter signals
    let filter = SignalFilter::new(model, 0.6); // 60% confidence threshold
    let filtered_signals = filter.filter(&signal_dataset.signals, &features)?;

    let impact = filter.analyze_impact(&signal_dataset.signals, &filtered_signals);
    println!("\nFiltering impact:");
    println!("  Original: {} signals", impact.original_count);
    println!("  Filtered: {} signals", impact.filtered_count);
    println!("  Retention: {:.1}%", impact.retention_rate);

    Ok(())
}
```

## Verification

### Test 1: Label Distribution Balance
```rust
#[test]
fn test_label_distribution() {
    let signal_dataset = create_balanced_test_dataset();
    let config = LabelingConfig::default();
    let labeler = TripleBarrierLabeler::new(config);

    let labeled = labeler.label(&signal_dataset).unwrap();
    let stats = TripleBarrierLabeler::analyze_distribution(&labeled);

    // Should have mix of all three labels
    assert!(stats.profit_count > 0);
    assert!(stats.loss_count > 0);
    assert!(stats.timeout_count > 0);

    // Total should match input
    assert_eq!(stats.total_count, signal_dataset.signals.len());
}
```

### Test 2: Barrier Logic
```rust
#[test]
fn test_barrier_hit_logic() {
    let signal = create_test_signal_at_index(100);
    let data = create_trending_data(); // Price goes up

    let config = LabelingConfig {
        profit_target_pct: 0.01,
        stop_loss_pct: 0.02,
        time_limit_bars: 20,
        ..Default::default()
    };

    let labeler = TripleBarrierLabeler::new(config);
    let labeled = labeler.label_single_signal(&signal, &data).unwrap();

    // With upward trend and small profit target, should hit profit
    assert_eq!(labeled.label, Label::Profit);
    assert_eq!(labeled.hit_barrier, BarrierType::Upper);
}
```

## Common Issues

### Issue: All labels are Timeout
**Solution**: Barriers are too wide. Reduce `profit_target_pct` and `stop_loss_pct`, or increase `time_limit_bars`.

### Issue: Imbalanced labels (95% one class)
**Solution**: Adjust barrier ratios. For trading, aim for risk:reward of 1:2 (stop_loss = 1%, profit = 2%).

### Issue: Model has poor accuracy
**Solution**: Check feature quality. May need more diverse features or better feature engineering.

## Next Steps

For configuration system, proceed to **[16-configuration-system.md](./16-configuration-system.md)**.
For data connectors, proceed to **[17-data-connectors.md](./17-data-connectors.md)**.
