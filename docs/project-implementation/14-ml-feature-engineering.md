# 14 - ML Feature Engineering & Signal Extraction

## Goal
Implement feature engineering for the ML meta-model that filters trading signals. This creates meaningful features from market data and strategy signals to predict signal quality.

## Prerequisites
- **07-backtesting-engine.md** - Strategy execution
- **11-evolution-engine.md** - Strategy generation
- **02-type-system.md** - Core types

## What You'll Create
1. `SignalExtractor` - Extract entry signals from strategy ASTs
2. `FeatureEngineer` - Create ML features from market data and signals
3. Feature types: Price action, volatility, momentum, volume, temporal
4. Lookahead bias prevention
5. Feature normalization and scaling

## Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│           Strategy AST                                  │
│     if RSI(Close, 14) > 70 then OpenLong               │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ↓
┌──────────────────────────────────────────────────────────┐
│         SignalExtractor                                   │
│  • Evaluate condition over full dataset                   │
│  • Extract signal timestamps and indicator values         │
│  • Output: DataFrame with signal bars                     │
└──────────────────┬───────────────────────────────────────┘
                   │
                   ↓
┌──────────────────────────────────────────────────────────┐
│         FeatureEngineer                                   │
│                                                           │
│  ┌────────────────────────────────────────────────┐     │
│  │ Price Features                                  │     │
│  │ • Returns (1-bar, 5-bar, 10-bar)               │     │
│  │ • Volatility (rolling std)                     │     │
│  │ • Distance from SMA                             │     │
│  └────────────────────────────────────────────────┘     │
│                                                           │
│  ┌────────────────────────────────────────────────┐     │
│  │ Momentum Features                               │     │
│  │ • RSI                                            │     │
│  │ • MACD                                           │     │
│  │ • Rate of change                                │     │
│  └────────────────────────────────────────────────┘     │
│                                                           │
│  ┌────────────────────────────────────────────────┐     │
│  │ Volume Features                                 │     │
│  │ • Volume ratio                                  │     │
│  │ • Volume moving average                        │     │
│  └────────────────────────────────────────────────┘     │
│                                                           │
│  ┌────────────────────────────────────────────────┐     │
│  │ Temporal Features                               │     │
│  │ • Hour of day                                   │     │
│  │ • Day of week                                   │     │
│  │ • Time since last signal                       │     │
│  └────────────────────────────────────────────────┘     │
│                                                           │
│  Output: Feature matrix (N_signals × N_features)         │
└──────────────────────────────────────────────────────────┘
```

## Key Concepts

### 1. Lookahead Bias Prevention

**BAD (Lookahead)**:
```rust
// Using future data to calculate features
let volatility_t = data[t..t+10].std(); // WRONG! Uses future data
```

**GOOD (No Lookahead)**:
```rust
// Using only past data
let volatility_t = data[t-10..t].std(); // Correct! Only past data
```

### 2. Feature Types

- **Stationary Features**: Returns, changes (better for ML)
- **Non-Stationary Features**: Prices, levels (avoid)
- **Normalized Features**: Z-scores, percentiles (scale-invariant)

### 3. Feature Windows

```
At signal time t, we can use:
├─ [t-20, t): Past 20 bars (OK)
├─ [t]: Current bar (OK with caution)
└─ [t+1, ...): Future bars (NEVER!)
```

## Implementation

### Step 1: Signal Data Structure

Create `src/ml/signals/types.rs`:

```rust
use polars::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub timestamp: DateTime<Utc>,
    pub bar_index: usize,
    pub direction: SignalDirection,
    pub indicator_values: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignalDirection {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct SignalDataset {
    pub signals: Vec<Signal>,
    pub market_data: DataFrame,
}
```

### Step 2: Signal Extractor

Create `src/ml/signals/extractor.rs`:

```rust
use super::types::*;
use crate::engines::generation::ast::StrategyAST;
use crate::engines::evaluation::expression::ExpressionBuilder;
use crate::error::TradeBiasError;
use polars::prelude::*;
use std::collections::HashMap;

pub struct SignalExtractor {
    expression_builder: ExpressionBuilder,
}

impl SignalExtractor {
    pub fn new(expression_builder: ExpressionBuilder) -> Self {
        Self { expression_builder }
    }

    /// Extract all signals from strategy AST
    pub fn extract(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<SignalDataset, TradeBiasError> {
        // Build condition expression
        let condition_expr = self.expression_builder.build_condition(ast)?;

        // Evaluate condition over full dataset
        let condition_series = data
            .lazy()
            .select([condition_expr])
            .collect()?
            .column("condition")?
            .bool()?
            .clone();

        // Find where condition is true
        let mut signals = Vec::new();

        for (idx, &is_signal) in condition_series.into_iter().enumerate() {
            if let Some(true) = is_signal {
                let timestamp = self.get_timestamp_at(data, idx)?;
                let direction = self.get_signal_direction(ast)?;
                let indicator_values = self.extract_indicator_values(data, idx)?;

                signals.push(Signal {
                    timestamp,
                    bar_index: idx,
                    direction,
                    indicator_values,
                });
            }
        }

        Ok(SignalDataset {
            signals,
            market_data: data.clone(),
        })
    }

    fn get_timestamp_at(&self, data: &DataFrame, idx: usize) -> Result<DateTime<Utc>, TradeBiasError> {
        let timestamp_ms = data
            .column("timestamp")?
            .datetime()?
            .get(idx)
            .ok_or_else(|| TradeBiasError::Validation("Invalid timestamp".to_string()))?;

        let timestamp_s = timestamp_ms / 1000;
        DateTime::<Utc>::from_timestamp(timestamp_s, 0)
            .ok_or_else(|| TradeBiasError::Validation("Invalid timestamp".to_string()))
    }

    fn get_signal_direction(&self, ast: &StrategyAST) -> Result<SignalDirection, TradeBiasError> {
        // Extract action from AST
        match ast {
            StrategyAST::Rule { action, .. } => {
                match action.as_ref() {
                    ASTNode::Call { function, .. } if function == "OpenLong" => Ok(SignalDirection::Long),
                    ASTNode::Call { function, .. } if function == "OpenShort" => Ok(SignalDirection::Short),
                    _ => Err(TradeBiasError::Validation("Unknown action".to_string())),
                }
            }
        }
    }

    fn extract_indicator_values(
        &self,
        data: &DataFrame,
        idx: usize,
    ) -> Result<HashMap<String, f64>, TradeBiasError> {
        let mut values = HashMap::new();

        // Extract common indicators at signal bar
        if let Ok(col) = data.column("close") {
            if let Some(val) = col.f64()?.get(idx) {
                values.insert("close".to_string(), val);
            }
        }

        // Add more indicators as needed...

        Ok(values)
    }
}
```

### Step 3: Feature Engineer

Create `src/ml/features/engineer.rs`:

```rust
use crate::ml::signals::types::*;
use crate::error::TradeBiasError;
use polars::prelude::*;

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
    ) -> Result<DataFrame, TradeBiasError> {
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
        DataFrame::new(feature_series).map_err(|e| TradeBiasError::Computation(e.to_string()))
    }

    fn create_price_features(
        &self,
        data: &DataFrame,
        signal_indices: &[usize],
    ) -> Result<Vec<Series>, TradeBiasError> {
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
    ) -> Result<Vec<Series>, TradeBiasError> {
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
    ) -> Result<Vec<Series>, TradeBiasError> {
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
    ) -> Result<Vec<Series>, TradeBiasError> {
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
    ) -> Result<Vec<Series>, TradeBiasError> {
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
```

## Usage Example

```rust
use tradebias::ml::signals::extractor::SignalExtractor;
use tradebias::ml::features::engineer::{FeatureEngineer, FeatureConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load strategy and data
    let ast = load_strategy_ast("strategy.json")?;
    let data = load_market_data("data.csv")?;

    // Extract signals
    let expression_builder = ExpressionBuilder::new(registry);
    let signal_extractor = SignalExtractor::new(expression_builder);
    let signal_dataset = signal_extractor.extract(&ast, &data)?;

    println!("Extracted {} signals", signal_dataset.signals.len());

    // Engineer features
    let config = FeatureConfig::default();
    let feature_engineer = FeatureEngineer::new(config);
    let features = feature_engineer.engineer(&signal_dataset)?;

    println!("Created features: {} rows × {} columns",
        features.height(), features.width());
    println!("Feature names: {:?}", features.get_column_names());

    Ok(())
}
```

## Verification

### Test 1: No Lookahead Bias
```rust
#[test]
fn test_no_lookahead() {
    let data = create_test_data_with_trend();
    let signal_dataset = create_test_signals(&data);

    let config = FeatureConfig::default();
    let engineer = FeatureEngineer::new(config);
    let features = engineer.engineer(&signal_dataset).unwrap();

    // Manually verify that return_10 feature uses only past data
    let return_10 = features.column("return_10").unwrap();

    // Feature at time t should not correlate with future price changes
    assert_no_future_correlation(&data, &return_10);
}
```

### Test 2: Feature Completeness
```rust
#[test]
fn test_feature_completeness() {
    let signal_dataset = create_test_signals_dataset();

    let config = FeatureConfig::default();
    let engineer = FeatureEngineer::new(config);
    let features = engineer.engineer(&signal_dataset).unwrap();

    // Should have all expected feature types
    assert!(features.column("return_5").is_ok());
    assert!(features.column("rsi_14").is_ok());
    assert!(features.column("volatility_10").is_ok());
    assert!(features.column("volume_ratio_20").is_ok());
    assert!(features.column("hour_of_day").is_ok());
}
```

## Common Issues

### Issue: Features contain NaN values
**Solution**: Check data quality at signal points. Ensure enough historical data exists for all windows.

### Issue: Features all have same value
**Solution**: Check signal extraction - might be extracting from constant regions. Add validation for signal diversity.

### Issue: Low correlation with labels
**Solution**: Try different feature types or windows. Not all features will be predictive - that's what feature selection is for (next doc).

## Next Steps

Proceed to **[15-ml-meta-labeling.md](./15-ml-meta-labeling.md)** to implement the triple-barrier labeling method and meta-model training.
