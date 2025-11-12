# 19 - Calibration Engine & Signal Extraction

## Goal
Implement auto-calibration of indicator parameter ranges and signal extraction utilities. This optimizes indicator parameters for specific datasets and provides clean signal extraction from strategy ASTs.

## Prerequisites
- **03-primitives.md** - Primitives implementation
- **04-06** - Indicators
- **11-evolution-engine.md** - Strategy representation
- **18-data-connectors.md** - Data loading

## What You'll Create
1. `CalibrationEngine` - Auto-calibrate indicator ranges for dataset
2. `SignalEngine` - Extract signals from strategy ASTs
3. Percentile-based parameter optimization
4. Dataset-specific parameter suggestions
5. Signal quality metrics

## Why Calibration?

**Problem**: Default indicator parameters (e.g., RSI period = 14) may not be optimal for all assets and timeframes.

**Solution**: Analyze the dataset to find optimal parameter ranges:
- RSI: Find periods that maximize signal separation
- ATR: Calibrate multipliers based on actual volatility
- Moving Averages: Determine effective lookback windows

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│           CalibrationEngine                           │
│                                                       │
│  Input: Market Data (DataFrame)                      │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ 1. Analyze Data Characteristics                │  │
│  │  • Volatility distribution                     │  │
│  │  • Price range                                 │  │
│  │  • Trend persistence                           │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ 2. Test Parameter Ranges                       │  │
│  │  • RSI: periods 9-25                           │  │
│  │  • ATR: multipliers 1-3                        │  │
│  │  • MA: periods 10-200                          │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ 3. Calculate Quality Metrics                   │  │
│  │  • Signal clarity (separation)                 │  │
│  │  • Frequency (not too rare/common)             │  │
│  │  • Stability (consistent behavior)             │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  Output: Optimal parameter ranges for this dataset   │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│           SignalEngine                                │
│                                                       │
│  Input: Strategy AST + Market Data                   │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ 1. Build Condition Expression                  │  │
│  │    from AST                                     │  │
│  └────────────────────────────────────────────────┘  │
│                      ↓                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ 2. Evaluate over Full Dataset                  │  │
│  │    (vectorized)                                 │  │
│  └────────────────────────────────────────────────┘  │
│                      ↓                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ 3. Extract Signal Bars                         │  │
│  │    where condition = true                       │  │
│  └────────────────────────────────────────────────┘  │
│                      ↓                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ 4. Enrich with Indicator Values                │  │
│  │    at signal time                               │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  Output: List of signals with context                │
└──────────────────────────────────────────────────────┘
```

## Implementation

### Step 1: Calibration Configuration

Create `src/engines/calibration/config.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationConfig {
    pub rsi_periods: Vec<usize>,
    pub ma_periods: Vec<usize>,
    pub atr_periods: Vec<usize>,
    pub atr_multipliers: Vec<f64>,
    pub min_signal_frequency_pct: f64,  // e.g., 1.0 = 1% of bars
    pub max_signal_frequency_pct: f64,  // e.g., 10.0 = 10% of bars
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            rsi_periods: vec![9, 10, 12, 14, 16, 18, 20, 21, 25],
            ma_periods: vec![5, 10, 14, 20, 25, 30, 50, 100, 200],
            atr_periods: vec![7, 10, 14, 20, 21],
            atr_multipliers: vec![1.0, 1.5, 2.0, 2.5, 3.0],
            min_signal_frequency_pct: 1.0,
            max_signal_frequency_pct: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub indicator: String,
    pub optimal_params: Vec<usize>,
    pub score: f64,
    pub signal_frequency_pct: f64,
    pub signal_clarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetProfile {
    pub volatility_mean: f64,
    pub volatility_std: f64,
    pub price_range_pct: f64,
    pub trend_strength: f64,
    pub calibration_results: Vec<CalibrationResult>,
}
```

### Step 2: Calibration Engine

Create `src/engines/calibration/engine.rs`:

```rust
use super::config::*;
use crate::functions::indicators::*;
use crate::error::TradeBiasError;
use polars::prelude::*;

pub struct CalibrationEngine {
    config: CalibrationConfig,
}

impl CalibrationEngine {
    pub fn new(config: CalibrationConfig) -> Self {
        Self { config }
    }

    pub fn calibrate(&self, data: &DataFrame) -> Result<DatasetProfile, TradeBiasError> {
        // Analyze dataset characteristics
        let volatility = self.calculate_volatility(data)?;
        let price_range = self.calculate_price_range(data)?;
        let trend_strength = self.calculate_trend_strength(data)?;

        // Calibrate indicators
        let mut calibration_results = Vec::new();

        // Calibrate RSI
        if let Ok(rsi_result) = self.calibrate_rsi(data) {
            calibration_results.push(rsi_result);
        }

        // Calibrate ATR
        if let Ok(atr_result) = self.calibrate_atr(data) {
            calibration_results.push(atr_result);
        }

        // Calibrate Moving Averages
        if let Ok(ma_result) = self.calibrate_moving_average(data) {
            calibration_results.push(ma_result);
        }

        Ok(DatasetProfile {
            volatility_mean: volatility.0,
            volatility_std: volatility.1,
            price_range_pct: price_range,
            trend_strength,
            calibration_results,
        })
    }

    fn calculate_volatility(&self, data: &DataFrame) -> Result<(f64, f64), TradeBiasError> {
        let close = data.column("close")?.f64()?;

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..close.len() {
            if let (Some(curr), Some(prev)) = (close.get(i), close.get(i - 1)) {
                if prev != 0.0 {
                    returns.push((curr - prev) / prev);
                }
            }
        }

        if returns.is_empty() {
            return Ok((0.0, 0.0));
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        let std = variance.sqrt();

        Ok((mean, std))
    }

    fn calculate_price_range(&self, data: &DataFrame) -> Result<f64, TradeBiasError> {
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;

        let max_high = high.max().unwrap_or(0.0);
        let min_low = low.min().unwrap_or(0.0);

        if min_low == 0.0 {
            return Ok(0.0);
        }

        let range_pct = ((max_high - min_low) / min_low) * 100.0;
        Ok(range_pct)
    }

    fn calculate_trend_strength(&self, data: &DataFrame) -> Result<f64, TradeBiasError> {
        let close = data.column("close")?.f64()?;

        // Calculate correlation with time
        let n = close.len() as f64;
        let time: Vec<f64> = (0..close.len()).map(|i| i as f64).collect();

        let mean_time = time.iter().sum::<f64>() / n;
        let mean_price = close.iter().filter_map(|p| *p).sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denom_time = 0.0;
        let mut denom_price = 0.0;

        for (i, &t) in time.iter().enumerate() {
            if let Some(p) = close.get(i) {
                numerator += (t - mean_time) * (p - mean_price);
                denom_time += (t - mean_time).powi(2);
                denom_price += (p - mean_price).powi(2);
            }
        }

        let correlation = if denom_time > 0.0 && denom_price > 0.0 {
            numerator / (denom_time * denom_price).sqrt()
        } else {
            0.0
        };

        Ok(correlation.abs())
    }

    fn calibrate_rsi(&self, data: &DataFrame) -> Result<CalibrationResult, TradeBiasError> {
        let mut best_score = 0.0;
        let mut best_period = 14;
        let mut best_frequency = 0.0;
        let mut best_clarity = 0.0;

        for &period in &self.config.rsi_periods {
            // Calculate RSI with this period
            let rsi_values = self.calculate_rsi_for_period(data, period)?;

            // Test oversold/overbought signals
            let oversold_signals = self.count_signals(&rsi_values, |v| v < 30.0);
            let overbought_signals = self.count_signals(&rsi_values, |v| v > 70.0);

            let total_signals = oversold_signals + overbought_signals;
            let signal_frequency = (total_signals as f64 / data.height() as f64) * 100.0;

            // Check if frequency is in acceptable range
            if signal_frequency < self.config.min_signal_frequency_pct
                || signal_frequency > self.config.max_signal_frequency_pct
            {
                continue;
            }

            // Calculate signal clarity (how well-separated signals are)
            let clarity = self.calculate_signal_clarity(&rsi_values)?;

            // Score: balance frequency and clarity
            let score = clarity * (1.0 - (signal_frequency - 5.0).abs() / 10.0);

            if score > best_score {
                best_score = score;
                best_period = period;
                best_frequency = signal_frequency;
                best_clarity = clarity;
            }
        }

        Ok(CalibrationResult {
            indicator: "RSI".to_string(),
            optimal_params: vec![best_period],
            score: best_score,
            signal_frequency_pct: best_frequency,
            signal_clarity: best_clarity,
        })
    }

    fn calibrate_atr(&self, data: &DataFrame) -> Result<CalibrationResult, TradeBiasError> {
        let mut best_score = 0.0;
        let mut best_period = 14;

        for &period in &self.config.atr_periods {
            let atr_values = self.calculate_atr_for_period(data, period)?;

            // Score based on stability (lower CV is better)
            let mean = atr_values.iter().filter_map(|&v| v).sum::<f64>() / atr_values.len() as f64;
            let variance = atr_values
                .iter()
                .filter_map(|&v| v)
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / (atr_values.len() - 1) as f64;

            let cv = variance.sqrt() / mean;
            let score = 1.0 / (1.0 + cv);

            if score > best_score {
                best_score = score;
                best_period = period;
            }
        }

        Ok(CalibrationResult {
            indicator: "ATR".to_string(),
            optimal_params: vec![best_period],
            score: best_score,
            signal_frequency_pct: 0.0, // N/A for ATR
            signal_clarity: 0.0,        // N/A
        })
    }

    fn calibrate_moving_average(&self, data: &DataFrame) -> Result<CalibrationResult, TradeBiasError> {
        let close = data.column("close")?.f64()?;
        let mut best_score = 0.0;
        let mut best_period = 20;

        for &period in &self.config.ma_periods {
            if period > close.len() {
                continue;
            }

            // Calculate SMA
            let sma = self.calculate_sma_for_period(data, period)?;

            // Count crossovers
            let mut crossovers = 0;
            for i in 1..close.len() {
                if let (Some(curr_close), Some(prev_close), Some(curr_sma), Some(prev_sma)) = (
                    close.get(i),
                    close.get(i - 1),
                    sma.get(i),
                    sma.get(i - 1),
                ) {
                    // Cross above
                    if prev_close <= prev_sma && curr_close > curr_sma {
                        crossovers += 1;
                    }
                    // Cross below
                    if prev_close >= prev_sma && curr_close < curr_sma {
                        crossovers += 1;
                    }
                }
            }

            let crossover_frequency = (crossovers as f64 / close.len() as f64) * 100.0;

            if crossover_frequency < self.config.min_signal_frequency_pct
                || crossover_frequency > self.config.max_signal_frequency_pct
            {
                continue;
            }

            // Score based on crossover frequency
            let score = 1.0 - (crossover_frequency - 5.0).abs() / 10.0;

            if score > best_score {
                best_score = score;
                best_period = period;
            }
        }

        Ok(CalibrationResult {
            indicator: "SMA".to_string(),
            optimal_params: vec![best_period],
            score: best_score,
            signal_frequency_pct: 0.0,
            signal_clarity: 0.0,
        })
    }

    // Helper methods
    fn calculate_rsi_for_period(
        &self,
        data: &DataFrame,
        period: usize,
    ) -> Result<Vec<Option<f64>>, TradeBiasError> {
        // Simplified RSI calculation
        let close = data.column("close")?.f64()?;
        let mut rsi_values = Vec::new();

        for i in 0..close.len() {
            if i < period + 1 {
                rsi_values.push(None);
                continue;
            }

            let mut gains = 0.0;
            let mut losses = 0.0;

            for j in (i - period)..i {
                if let (Some(curr), Some(prev)) = (close.get(j), close.get(j - 1)) {
                    let change = curr - prev;
                    if change > 0.0 {
                        gains += change;
                    } else {
                        losses += -change;
                    }
                }
            }

            let avg_gain = gains / period as f64;
            let avg_loss = losses / period as f64;

            let rsi = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
            };

            rsi_values.push(Some(rsi));
        }

        Ok(rsi_values)
    }

    fn calculate_atr_for_period(
        &self,
        data: &DataFrame,
        period: usize,
    ) -> Result<Vec<Option<f64>>, TradeBiasError> {
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let close = data.column("close")?.f64()?;

        let mut atr_values = Vec::new();

        for i in 0..high.len() {
            if i < period {
                atr_values.push(None);
                continue;
            }

            let mut tr_sum = 0.0;
            for j in (i - period)..i {
                if let (Some(h), Some(l), Some(prev_c)) = (
                    high.get(j),
                    low.get(j),
                    close.get(j.saturating_sub(1)),
                ) {
                    let tr = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
                    tr_sum += tr;
                }
            }

            atr_values.push(Some(tr_sum / period as f64));
        }

        Ok(atr_values)
    }

    fn calculate_sma_for_period(
        &self,
        data: &DataFrame,
        period: usize,
    ) -> Result<Vec<Option<f64>>, TradeBiasError> {
        let close = data.column("close")?.f64()?;
        let mut sma_values = Vec::new();

        for i in 0..close.len() {
            if i < period {
                sma_values.push(None);
                continue;
            }

            let mut sum = 0.0;
            let mut count = 0;

            for j in (i - period)..i {
                if let Some(val) = close.get(j) {
                    sum += val;
                    count += 1;
                }
            }

            if count > 0 {
                sma_values.push(Some(sum / count as f64));
            } else {
                sma_values.push(None);
            }
        }

        Ok(sma_values)
    }

    fn count_signals<F>(&self, values: &[Option<f64>], predicate: F) -> usize
    where
        F: Fn(f64) -> bool,
    {
        values
            .iter()
            .filter_map(|&v| v)
            .filter(|&v| predicate(v))
            .count()
    }

    fn calculate_signal_clarity(&self, values: &[Option<f64>]) -> Result<f64, TradeBiasError> {
        // Simplified: measure variance in signal region vs neutral region
        let signal_values: Vec<f64> = values
            .iter()
            .filter_map(|&v| v)
            .filter(|&v| v < 30.0 || v > 70.0)
            .collect();

        let neutral_values: Vec<f64> = values
            .iter()
            .filter_map(|&v| v)
            .filter(|&v| v >= 30.0 && v <= 70.0)
            .collect();

        if signal_values.is_empty() || neutral_values.is_empty() {
            return Ok(0.0);
        }

        let signal_mean = signal_values.iter().sum::<f64>() / signal_values.len() as f64;
        let neutral_mean = neutral_values.iter().sum::<f64>() / neutral_values.len() as f64;

        let separation = (signal_mean - neutral_mean).abs();
        let clarity = separation / 50.0; // Normalize to 0-1

        Ok(clarity.min(1.0))
    }
}
```

### Step 3: Signal Engine

Create `src/engines/signal_engine.rs`:

```rust
use crate::engines::evaluation::expression::ExpressionBuilder;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedSignal {
    pub bar_index: usize,
    pub timestamp: DateTime<Utc>,
    pub signal_type: SignalType,
    pub indicator_values: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignalType {
    Entry,
    Exit,
}

pub struct SignalEngine {
    expression_builder: ExpressionBuilder,
}

impl SignalEngine {
    pub fn new(expression_builder: ExpressionBuilder) -> Self {
        Self { expression_builder }
    }

    pub fn extract_signals(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<Vec<ExtractedSignal>, TradeBiasError> {
        // Build condition expression
        let condition_expr = self.expression_builder.build_condition(ast)?;

        // Evaluate over dataset
        let condition_series = data
            .lazy()
            .select([condition_expr.alias("signal")])
            .collect()?
            .column("signal")?
            .bool()?
            .clone();

        // Extract signal bars
        let timestamps = data.column("timestamp")?.datetime()?;
        let mut signals = Vec::new();

        for (idx, is_signal) in condition_series.into_iter().enumerate() {
            if is_signal == Some(true) {
                let timestamp_ms = timestamps.get(idx).ok_or_else(|| {
                    TradeBiasError::Validation("Invalid timestamp".to_string())
                })?;

                let timestamp = DateTime::from_timestamp(timestamp_ms / 1000, 0)
                    .ok_or_else(|| TradeBiasError::Validation("Invalid timestamp".to_string()))?;

                // Extract indicator values at this bar
                let indicator_values = self.extract_indicator_values(data, idx)?;

                signals.push(ExtractedSignal {
                    bar_index: idx,
                    timestamp,
                    signal_type: SignalType::Entry,
                    indicator_values,
                });
            }
        }

        Ok(signals)
    }

    fn extract_indicator_values(
        &self,
        data: &DataFrame,
        idx: usize,
    ) -> Result<std::collections::HashMap<String, f64>, TradeBiasError> {
        let mut values = std::collections::HashMap::new();

        // Extract OHLCV
        for col_name in &["open", "high", "low", "close", "volume"] {
            if let Ok(col) = data.column(col_name) {
                if let Some(val) = col.f64()?.get(idx) {
                    values.insert(col_name.to_string(), val);
                }
            }
        }

        Ok(values)
    }

    pub fn calculate_signal_quality(&self, signals: &[ExtractedSignal]) -> SignalQualityMetrics {
        SignalQualityMetrics {
            total_signals: signals.len(),
            signals_per_100_bars: (signals.len() as f64 / 100.0) * 100.0,
            // Add more metrics as needed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalQualityMetrics {
    pub total_signals: usize,
    pub signals_per_100_bars: f64,
}
```

## Usage Example

```rust
use tradebias::engines::calibration::{CalibrationEngine, CalibrationConfig};
use tradebias::engines::signal_engine::SignalEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let data = load_market_data("data.csv")?;

    // 1. Calibrate indicators
    let calibration_config = CalibrationConfig::default();
    let calibration_engine = CalibrationEngine::new(calibration_config);

    let profile = calibration_engine.calibrate(&data)?;

    println!("Dataset Profile:");
    println!("  Volatility: {:.4} ± {:.4}",
        profile.volatility_mean, profile.volatility_std);
    println!("  Price Range: {:.2}%", profile.price_range_pct);
    println!("  Trend Strength: {:.2}", profile.trend_strength);

    println!("\nCalibration Results:");
    for result in &profile.calibration_results {
        println!("  {}: params={:?}, score={:.2}",
            result.indicator, result.optimal_params, result.score);
    }

    // 2. Extract signals
    let ast = load_strategy_ast("strategy.json")?;
    let expression_builder = ExpressionBuilder::new(registry);
    let signal_engine = SignalEngine::new(expression_builder);

    let signals = signal_engine.extract_signals(&ast, &data)?;
    println!("\nExtracted {} signals", signals.len());

    let quality = signal_engine.calculate_signal_quality(&signals);
    println!("Signal quality: {:.2} signals per 100 bars",
        quality.signals_per_100_bars);

    Ok(())
}
```

## Verification

### Test 1: Calibration
```rust
#[test]
fn test_calibration() {
    let data = create_test_data_with_trends();
    let config = CalibrationConfig::default();
    let engine = CalibrationEngine::new(config);

    let profile = engine.calibrate(&data).unwrap();

    // Should find optimal RSI period
    let rsi_result = profile.calibration_results
        .iter()
        .find(|r| r.indicator == "RSI")
        .unwrap();

    assert!(rsi_result.optimal_params[0] >= 9);
    assert!(rsi_result.optimal_params[0] <= 25);
}
```

### Test 2: Signal Extraction
```rust
#[test]
fn test_signal_extraction() {
    let ast = create_simple_strategy();
    let data = create_test_data();

    let engine = SignalEngine::new(expression_builder);
    let signals = engine.extract_signals(&ast, &data).unwrap();

    assert!(!signals.is_empty());

    // All signals should have valid timestamps
    for signal in &signals {
        assert!(signal.bar_index < data.height());
    }
}
```

## Common Issues

### Issue: Calibration finds no good parameters
**Solution**: Dataset may not suit the indicator. Try different indicators or adjust frequency thresholds.

### Issue: Signal extraction returns empty
**Solution**: Condition never evaluates to true. Check AST validity and data compatibility.

## Next Steps

You now have comprehensive instruction files covering all major components! Update the overview document to include files 16-19.
