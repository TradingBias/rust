use crate::ml::signals::types::*;
use crate::error::TradebiasError;
use polars::prelude::*;
use chrono::{Datelike, Timelike};

pub struct FeatureConfig {
    pub price_features: bool,
    pub momentum_features: bool,
    pub volatility_features: bool,
    pub volume_features: bool,
    pub temporal_features: bool,
    pub lookback_windows: Vec<usize>, // e.g., [5, 10, 20]
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            price_features: true,
            momentum_features: true,
            volatility_features: true,
            volume_features: true,
            temporal_features: true,
            lookback_windows: vec![5, 10, 20],
        }
    }
}

pub struct FeatureEngineer {
    config: FeatureConfig,
}

impl FeatureEngineer {
    pub fn new(config: FeatureConfig) -> Self {
        Self { config }
    }

    /// Create features for all signals
    pub fn engineer(
        &self,
        signal_dataset: &SignalDataset,
    ) -> Result<DataFrame, TradebiasError> {
        let mut feature_series: Vec<Series> = Vec::new();

        // Signal indices
        let signal_indices: Vec<usize> = signal_dataset
            .signals
            .iter()
            .map(|s| s.bar_index)
            .collect();

        // Price features
        if self.config.price_features {
            feature_series.extend(self.create_price_features(&signal_dataset.market_data, &signal_indices)?);
        }

        // Momentum features
        if self.config.momentum_features {
            feature_series.extend(self.create_momentum_features(&signal_dataset.market_data, &signal_indices)?);
        }

        // Volatility features
        if self.config.volatility_features {
            feature_series.extend(self.create_volatility_features(&signal_dataset.market_data, &signal_indices)?);
        }

        // Volume features
        if self.config.volume_features {
            feature_series.extend(self.create_volume_features(&signal_dataset.market_data, &signal_indices)?);
        }

        // Temporal features
        if self.config.temporal_features {
            feature_series.extend(self.create_temporal_features(&signal_dataset.market_data, &signal_indices)?);
        }

        // Combine into DataFrame
        DataFrame::new(feature_series).map_err(|e| TradebiasError::Computation(e.to_string()))
    }

    fn create_price_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradebiasError> {
        let close = data.column("close")?.f64()?;
        let mut features = Vec::new();

        // Returns over different windows (NO LOOKAHEAD!)
        for &window in &self.config.lookback_windows {
            let mut returns = Vec::new();

            for &idx in signal_indices {
                if idx >= window {
                    let current = close.get(idx).unwrap_or(0.0);
                    let past = close.get(idx - window).unwrap_or(current);
                    let ret = if past != 0.0 {
                        (current - past) / past
                    } else {
                        0.0
                    };
                    returns.push(ret);
                } else {
                    returns.push(0.0); // Not enough history
                }
            }

            let series = Series::new(&format!("return_{}", window), returns);
            features.push(series);
        }

        // Distance from moving average
        for &window in &self.config.lookback_windows {
            let mut distances = Vec::new();

            for &idx in signal_indices {
                if idx >= window {
                    let current = close.get(idx).unwrap_or(0.0);
                    let sma = self.calculate_sma(close, idx, window);
                    let distance = if sma != 0.0 {
                        (current - sma) / sma
                    } else {
                        0.0
                    };
                    distances.push(distance);
                } else {
                    distances.push(0.0);
                }
            }

            let series = Series::new(&format!("distance_sma_{}", window), distances);
            features.push(series);
        }

        Ok(features)
    }

    fn create_momentum_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradebiasError> {
        let close = data.column("close")?.f64()?;
        let mut features = Vec::new();

        // RSI-like feature
        let mut rsi_values = Vec::new();

        for &idx in signal_indices {
            let window = 14;
            if idx >= window {
                let rsi = self.calculate_rsi(close, idx, window);
                rsi_values.push(rsi);
            } else {
                rsi_values.push(50.0); // Neutral
            }
        }

        features.push(Series::new("rsi_14", rsi_values));

        // Rate of change
        for &window in &self.config.lookback_windows {
            let mut roc_values = Vec::new();

            for &idx in signal_indices {
                if idx >= window {
                    let current = close.get(idx).unwrap_or(0.0);
                    let past = close.get(idx - window).unwrap_or(current);
                    let roc = if past != 0.0 {
                        ((current - past) / past) * 100.0
                    } else {
                        0.0
                    };
                    roc_values.push(roc);
                } else {
                    roc_values.push(0.0);
                }
            }

            features.push(Series::new(&format!("roc_{}", window), roc_values));
        }

        Ok(features)
    }

    fn create_volatility_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradebiasError> {
        let close = data.column("close")?.f64()?;
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let mut features = Vec::new();

        // Rolling standard deviation
        for &window in &self.config.lookback_windows {
            let mut vol_values = Vec::new();

            for &idx in signal_indices {
                if idx >= window {
                    let vol = self.calculate_std(close, idx, window);
                    vol_values.push(vol);
                } else {
                    vol_values.push(0.0);
                }
            }

            features.push(Series::new(&format!("volatility_{}", window), vol_values));
        }

        // Average True Range (ATR)
        let atr_window = 14;
        let mut atr_values = Vec::new();

        for &idx in signal_indices {
            if idx >= atr_window {
                let atr = self.calculate_atr(high, low, close, idx, atr_window);
                atr_values.push(atr);
            } else {
                atr_values.push(0.0);
            }
        }

        features.push(Series::new("atr_14", atr_values));

        Ok(features)
    }

    fn create_volume_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradebiasError> {
        let volume = data.column("volume")?.f64()?;
        let mut features = Vec::new();

        // Volume ratio (current / average)
        for &window in &self.config.lookback_windows {
            let mut ratios = Vec::new();

            for &idx in signal_indices {
                if idx >= window {
                    let current_vol = volume.get(idx).unwrap_or(0.0);
                    let avg_vol = self.calculate_mean(volume, idx, window);
                    let ratio = if avg_vol != 0.0 {
                        current_vol / avg_vol
                    } else {
                        1.0
                    };
                    ratios.push(ratio);
                } else {
                    ratios.push(1.0);
                }
            }

            features.push(Series::new(&format!("volume_ratio_{}", window), ratios));
        }

        Ok(features)
    }

    fn create_temporal_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradebiasError> {
        let timestamps = data.column("timestamp")?.datetime()?;
        let mut features = Vec::new();

        let mut hours = Vec::new();
        let mut days_of_week = Vec::new();

        for &idx in signal_indices {
            if let Some(ts_ms) = timestamps.get(idx) {
                let ts_s = ts_ms / 1000;
                if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(ts_s, 0) {
                    hours.push(dt.hour() as f64);
                    days_of_week.push(dt.weekday().num_days_from_monday() as f64);
                } else {
                    hours.push(0.0);
                    days_of_week.push(0.0);
                }
            } else {
                hours.push(0.0);
                days_of_week.push(0.0);
            }
        }

        features.push(Series::new("hour_of_day", hours));
        features.push(Series::new("day_of_week", days_of_week));

        Ok(features)
    }

    // Helper methods
    fn calculate_sma(&self, series: &Float64Chunked, idx: usize, window: usize) -> f64 {
        self.calculate_mean(series, idx, window)
    }

    fn calculate_mean(&self, series: &Float64Chunked, idx: usize, window: usize) -> f64 {
        let start = if idx >= window { idx - window } else { 0 };
        let mut sum = 0.0;
        let mut count = 0;

        for i in start..idx {
            if let Some(val) = series.get(i) {
                sum += val;
                count += 1;
            }
        }

        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }

    fn calculate_std(&self, series: &Float64Chunked, idx: usize, window: usize) -> f64 {
        let mean = self.calculate_mean(series, idx, window);
        let start = if idx >= window { idx - window } else { 0 };
        let mut sum_sq_diff = 0.0;
        let mut count = 0;

        for i in start..idx {
            if let Some(val) = series.get(i) {
                sum_sq_diff += (val - mean).powi(2);
                count += 1;
            }
        }

        if count > 1 {
            (sum_sq_diff / (count - 1) as f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_rsi(&self, series: &Float64Chunked, idx: usize, window: usize) -> f64 {
        if idx < window + 1 {
            return 50.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (idx - window)..idx {
            if let (Some(current), Some(prev)) = (series.get(i), series.get(i - 1)) {
                let change = current - prev;
                if change > 0.0 {
                    gains += change;
                } else {
                    losses += -change;
                }
            }
        }

        let avg_gain = gains / window as f64;
        let avg_loss = losses / window as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    fn calculate_atr(
        &self,
        high: &Float64Chunked,
        low: &Float64Chunked,
        close: &Float64Chunked,
        idx: usize,
        window: usize,
    ) -> f64 {
        if idx < window + 1 {
            return 0.0;
        }

        let mut tr_sum = 0.0;

        for i in (idx - window)..idx {
            if let (Some(h), Some(l), Some(prev_c)) = (high.get(i), low.get(i), close.get(i - 1)) {
                let tr = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
                tr_sum += tr;
            }
        }

        tr_sum / window as f64
    }
}
