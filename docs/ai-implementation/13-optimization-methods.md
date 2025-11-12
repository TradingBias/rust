# 13 - Optimization Methods & Walk-Forward Testing

## Goal
Implement validation methods for robust out-of-sample testing, including Walk-Forward Optimization (WFO) and simple train/test splits. This prevents overfitting and tests strategy adaptability.

## Prerequisites
- **07-backtesting-engine.md** - Backtesting infrastructure
- **08-metrics-engine.md** - Performance metrics
- **11-evolution-engine.md** - Evolution process

## What You'll Create
1. Data splitting strategies (Simple, Walk-Forward)
2. Validation methods for in-sample/out-of-sample testing
3. WFO result aggregation and analysis
4. Anchored vs sliding window implementations

## Core Concept: Walk-Forward Optimization

**Problem**: Strategies optimized on historical data often fail in live trading due to overfitting.

**Solution**: WFO simulates live trading by:
1. Splitting data into multiple windows
2. Training on in-sample (IS) period
3. Testing on out-of-sample (OOS) period
4. Rolling the window forward
5. Aggregating results

```
Data Timeline:
├────────────────────────────────────────────────────────┤
│  Window 1       │  Window 2       │  Window 3         │
├─────────┬───────┼─────────┬───────┼─────────┬─────────┤
│   IS    │  OOS  │   IS    │  OOS  │   IS    │  OOS    │
│  Train  │ Test  │  Train  │ Test  │  Train  │  Test   │
└─────────┴───────┴─────────┴───────┴─────────┴─────────┘

IS = In-Sample (70%), OOS = Out-of-Sample (30%)
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│            ValidationMethod (Trait)                     │
├─────────────────────────────────────────────────────────┤
│  • split_data()                                         │
│  • validate()                                           │
│  • aggregate_results()                                  │
└────────────────┬────────────────────────────────────────┘
                 │
      ┌──────────┴──────────┐
      │                     │
┌─────▼──────┐       ┌──────▼────────┐
│  Simple    │       │  Walk-Forward │
│  Method    │       │  Method       │
├────────────┤       ├───────────────┤
│ • Single   │       │ • Multiple    │
│   split    │       │   windows     │
│ • IS/OOS   │       │ • Rolling     │
└────────────┘       │ • Anchored    │
                     └───────────────┘
```

## Implementation

### Step 1: Data Split Types

Create `src/engines/generation/optimisation/splitters/types.rs`:

```rust
use polars::prelude::*;
use chrono::{DateTime, Utc};

/// Single data split (in-sample + out-of-sample)
#[derive(Debug, Clone)]
pub struct DataSplit {
    pub in_sample: DataFrame,
    pub out_of_sample: DataFrame,
    pub fold_num: usize,
    pub in_sample_start: DateTime<Utc>,
    pub in_sample_end: DateTime<Utc>,
    pub out_of_sample_start: DateTime<Utc>,
    pub out_of_sample_end: DateTime<Utc>,
}

/// Configuration for data splitting
#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub in_sample_pct: f64,    // e.g., 0.7 = 70% IS
    pub out_of_sample_pct: f64, // e.g., 0.3 = 30% OOS
    pub n_folds: usize,         // Number of WFO windows
    pub window_type: WindowType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowType {
    Sliding,  // Each window uses fixed-size IS period
    Anchored, // IS period grows with each window
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            in_sample_pct: 0.7,
            out_of_sample_pct: 0.3,
            n_folds: 5,
            window_type: WindowType::Sliding,
        }
    }
}
```

### Step 2: Base Splitter Trait

Create `src/engines/generation/optimisation/splitters/base.rs`:

```rust
use super::types::*;
use polars::prelude::*;
use crate::error::TradeBiasError;

pub trait DataSplitter: Send + Sync {
    /// Split data into multiple folds
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradeBiasError>;

    /// Get splitter configuration
    fn config(&self) -> &SplitConfig;
}
```

### Step 3: Simple Splitter

Create `src/engines/generation/optimisation/splitters/simple.rs`:

```rust
use super::base::DataSplitter;
use super::types::*;
use crate::error::TradeBiasError;
use polars::prelude::*;
use chrono::{DateTime, Utc};

pub struct SimpleSplitter {
    config: SplitConfig,
}

impl SimpleSplitter {
    pub fn new(in_sample_pct: f64) -> Self {
        Self {
            config: SplitConfig {
                in_sample_pct,
                out_of_sample_pct: 1.0 - in_sample_pct,
                n_folds: 1,
                window_type: WindowType::Sliding,
            },
        }
    }
}

impl DataSplitter for SimpleSplitter {
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradeBiasError> {
        let total_rows = data.height();
        let is_rows = (total_rows as f64 * self.config.in_sample_pct) as usize;

        if is_rows == 0 || is_rows >= total_rows {
            return Err(TradeBiasError::Validation(
                "Invalid split: in-sample size is 0 or exceeds data size".to_string(),
            ));
        }

        // Split data
        let in_sample = data.slice(0, is_rows);
        let out_of_sample = data.slice(is_rows as i64, total_rows - is_rows);

        // Extract timestamps
        let timestamps = data.column("timestamp")?.datetime()?;
        let is_start = get_datetime_at_index(timestamps, 0)?;
        let is_end = get_datetime_at_index(timestamps, is_rows - 1)?;
        let oos_start = get_datetime_at_index(timestamps, is_rows)?;
        let oos_end = get_datetime_at_index(timestamps, total_rows - 1)?;

        Ok(vec![DataSplit {
            in_sample,
            out_of_sample,
            fold_num: 0,
            in_sample_start: is_start,
            in_sample_end: is_end,
            out_of_sample_start: oos_start,
            out_of_sample_end: oos_end,
        }])
    }

    fn config(&self) -> &SplitConfig {
        &self.config
    }
}

fn get_datetime_at_index(series: &DatetimeChunked, idx: usize) -> Result<DateTime<Utc>, TradeBiasError> {
    let timestamp_ms = series.get(idx).ok_or_else(|| {
        TradeBiasError::Validation(format!("Cannot get timestamp at index {}", idx))
    })?;

    let timestamp_s = timestamp_ms / 1000;
    let datetime = DateTime::<Utc>::from_timestamp(timestamp_s, 0).ok_or_else(|| {
        TradeBiasError::Validation(format!("Invalid timestamp: {}", timestamp_ms))
    })?;

    Ok(datetime)
}
```

### Step 4: Walk-Forward Splitter

Create `src/engines/generation/optimisation/splitters/wfo.rs`:

```rust
use super::base::DataSplitter;
use super::types::*;
use super::simple::get_datetime_at_index;
use crate::error::TradeBiasError;
use polars::prelude::*;

pub struct WalkForwardSplitter {
    config: SplitConfig,
}

impl WalkForwardSplitter {
    pub fn new(
        in_sample_pct: f64,
        out_of_sample_pct: f64,
        n_folds: usize,
        window_type: WindowType,
    ) -> Self {
        Self {
            config: SplitConfig {
                in_sample_pct,
                out_of_sample_pct,
                n_folds,
                window_type,
            },
        }
    }
}

impl DataSplitter for WalkForwardSplitter {
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradeBiasError> {
        let total_rows = data.height();
        let timestamps = data.column("timestamp")?.datetime()?;

        match self.config.window_type {
            WindowType::Sliding => self.split_sliding(data, total_rows, timestamps),
            WindowType::Anchored => self.split_anchored(data, total_rows, timestamps),
        }
    }

    fn config(&self) -> &SplitConfig {
        &self.config
    }
}

impl WalkForwardSplitter {
    /// Sliding window: Each fold has same IS size
    fn split_sliding(
        &self,
        data: &DataFrame,
        total_rows: usize,
        timestamps: &DatetimeChunked,
    ) -> Result<Vec<DataSplit>, TradeBiasError> {
        let window_size = total_rows / (self.config.n_folds + 1);
        let is_size = (window_size as f64 * self.config.in_sample_pct) as usize;
        let oos_size = window_size - is_size;

        let mut splits = Vec::new();

        for fold in 0..self.config.n_folds {
            let start_idx = fold * window_size;
            let is_end_idx = start_idx + is_size;
            let oos_end_idx = is_end_idx + oos_size;

            if oos_end_idx > total_rows {
                break; // Not enough data for this fold
            }

            let in_sample = data.slice(start_idx as i64, is_size);
            let out_of_sample = data.slice(is_end_idx as i64, oos_size);

            splits.push(DataSplit {
                in_sample,
                out_of_sample,
                fold_num: fold,
                in_sample_start: get_datetime_at_index(timestamps, start_idx)?,
                in_sample_end: get_datetime_at_index(timestamps, is_end_idx - 1)?,
                out_of_sample_start: get_datetime_at_index(timestamps, is_end_idx)?,
                out_of_sample_end: get_datetime_at_index(timestamps, oos_end_idx - 1)?,
            });
        }

        Ok(splits)
    }

    /// Anchored window: IS period grows with each fold
    fn split_anchored(
        &self,
        data: &DataFrame,
        total_rows: usize,
        timestamps: &DatetimeChunked,
    ) -> Result<Vec<DataSplit>, TradeBiasError> {
        let oos_size = total_rows / (self.config.n_folds + 1);

        let mut splits = Vec::new();

        for fold in 0..self.config.n_folds {
            let oos_start_idx = (fold + 1) * oos_size;
            let oos_end_idx = oos_start_idx + oos_size;

            if oos_end_idx > total_rows {
                break;
            }

            // IS grows: from start to OOS start
            let in_sample = data.slice(0, oos_start_idx);
            let out_of_sample = data.slice(oos_start_idx as i64, oos_size);

            splits.push(DataSplit {
                in_sample,
                out_of_sample,
                fold_num: fold,
                in_sample_start: get_datetime_at_index(timestamps, 0)?,
                in_sample_end: get_datetime_at_index(timestamps, oos_start_idx - 1)?,
                out_of_sample_start: get_datetime_at_index(timestamps, oos_start_idx)?,
                out_of_sample_end: get_datetime_at_index(timestamps, oos_end_idx - 1)?,
            });
        }

        Ok(splits)
    }
}
```

### Step 5: Validation Methods

Create `src/engines/generation/optimisation/methods/base.rs`:

```rust
use crate::data::types::StrategyResult;
use crate::engines::generation::ast::StrategyAST;
use crate::engines::generation::optimisation::splitters::base::DataSplitter;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub in_sample_result: StrategyResult,
    pub out_of_sample_result: StrategyResult,
    pub fold_num: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub method: String,
    pub folds: Vec<ValidationResult>,
    pub aggregate_metrics: HashMap<String, f64>,
}

pub trait ValidationMethod: Send + Sync {
    fn name(&self) -> &str;

    /// Validate strategy using this method
    fn validate(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<AggregatedResult, TradeBiasError>;
}
```

Create `src/engines/generation/optimisation/methods/wfo.rs`:

```rust
use super::base::*;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::engines::generation::optimisation::splitters::{
    base::DataSplitter,
    wfo::WalkForwardSplitter,
};
use crate::error::TradeBiasError;
use polars::prelude::*;
use std::collections::HashMap;

pub struct WalkForwardMethod {
    splitter: WalkForwardSplitter,
    backtester: Backtester,
}

impl WalkForwardMethod {
    pub fn new(
        in_sample_pct: f64,
        out_of_sample_pct: f64,
        n_folds: usize,
        window_type: WindowType,
        backtester: Backtester,
    ) -> Self {
        Self {
            splitter: WalkForwardSplitter::new(
                in_sample_pct,
                out_of_sample_pct,
                n_folds,
                window_type,
            ),
            backtester,
        }
    }
}

impl ValidationMethod for WalkForwardMethod {
    fn name(&self) -> &str {
        "Walk-Forward Optimization"
    }

    fn validate(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<AggregatedResult, TradeBiasError> {
        // Split data into folds
        let splits = self.splitter.split(data)?;

        // Run backtest on each fold
        let mut fold_results = Vec::new();

        for split in splits {
            // In-sample backtest
            let is_result = self.backtester.run(ast, &split.in_sample)?;

            // Out-of-sample backtest
            let oos_result = self.backtester.run(ast, &split.out_of_sample)?;

            fold_results.push(ValidationResult {
                in_sample_result: is_result,
                out_of_sample_result: oos_result,
                fold_num: split.fold_num,
            });
        }

        // Aggregate metrics across folds
        let aggregate_metrics = self.aggregate_metrics(&fold_results);

        Ok(AggregatedResult {
            method: self.name().to_string(),
            folds: fold_results,
            aggregate_metrics,
        })
    }
}

impl WalkForwardMethod {
    fn aggregate_metrics(&self, folds: &[ValidationResult]) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();

        if folds.is_empty() {
            return aggregated;
        }

        // Get metric names from first fold
        let metric_names: Vec<String> = folds[0]
            .out_of_sample_result
            .metrics
            .keys()
            .cloned()
            .collect();

        // Calculate mean of each metric across OOS results
        for metric_name in metric_names {
            let values: Vec<f64> = folds
                .iter()
                .filter_map(|f| f.out_of_sample_result.metrics.get(&metric_name).copied())
                .collect();

            if !values.is_empty() {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let std = calculate_std(&values, mean);

                aggregated.insert(format!("{}_mean", metric_name), mean);
                aggregated.insert(format!("{}_std", metric_name), std);
                aggregated.insert(format!("{}_min", metric_name), values.iter().copied().fold(f64::INFINITY, f64::min));
                aggregated.insert(format!("{}_max", metric_name), values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            }
        }

        // Calculate consistency score (lower std = more consistent)
        if let Some(sharpe_std) = aggregated.get("sharpe_ratio_std") {
            let consistency = 1.0 / (1.0 + sharpe_std);
            aggregated.insert("consistency_score".to_string(), consistency);
        }

        aggregated
    }
}

fn calculate_std(values: &[f64], mean: f64) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let variance = values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;

    variance.sqrt()
}
```

## Usage Example

```rust
use tradebias::engines::generation::optimisation::{
    methods::wfo::WalkForwardMethod,
    splitters::types::WindowType,
};
use tradebias::engines::evaluation::Backtester;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let data = load_market_data("data.csv")?;

    // Create backtester
    let backtester = Backtester::new(/* ... */);

    // Create WFO method
    let wfo = WalkForwardMethod::new(
        0.7,  // 70% in-sample
        0.3,  // 30% out-of-sample
        5,    // 5 folds
        WindowType::Sliding,
        backtester,
    );

    // Load strategy
    let ast = load_strategy_ast("strategy.json")?;

    // Run validation
    let result = wfo.validate(&ast, &data)?;

    // Print results
    println!("Method: {}", result.method);
    println!("\nPer-Fold Results:");
    for fold_result in &result.folds {
        println!("  Fold {}: OOS Sharpe = {:.2}",
            fold_result.fold_num,
            fold_result.out_of_sample_result.metrics.get("sharpe_ratio").unwrap_or(&0.0)
        );
    }

    println!("\nAggregate Metrics:");
    println!("  Sharpe (mean): {:.2}", result.aggregate_metrics.get("sharpe_ratio_mean").unwrap_or(&0.0));
    println!("  Sharpe (std): {:.2}", result.aggregate_metrics.get("sharpe_ratio_std").unwrap_or(&0.0));
    println!("  Consistency: {:.2}", result.aggregate_metrics.get("consistency_score").unwrap_or(&0.0));

    Ok(())
}
```

## Verification

### Test 1: Simple Split
```rust
#[test]
fn test_simple_split() {
    let data = create_test_data(1000);
    let splitter = SimpleSplitter::new(0.7);

    let splits = splitter.split(&data).unwrap();

    assert_eq!(splits.len(), 1);
    assert_eq!(splits[0].in_sample.height(), 700);
    assert_eq!(splits[0].out_of_sample.height(), 300);
}
```

### Test 2: WFO Sliding Window
```rust
#[test]
fn test_wfo_sliding() {
    let data = create_test_data(1000);
    let splitter = WalkForwardSplitter::new(0.7, 0.3, 5, WindowType::Sliding);

    let splits = splitter.split(&data).unwrap();

    assert_eq!(splits.len(), 5);

    // All IS windows should be same size
    let is_sizes: Vec<_> = splits.iter().map(|s| s.in_sample.height()).collect();
    assert!(is_sizes.windows(2).all(|w| w[0] == w[1]));
}
```

### Test 3: WFO Anchored Window
```rust
#[test]
fn test_wfo_anchored() {
    let data = create_test_data(1000);
    let splitter = WalkForwardSplitter::new(0.7, 0.3, 5, WindowType::Anchored);

    let splits = splitter.split(&data).unwrap();

    assert_eq!(splits.len(), 5);

    // IS windows should grow
    let is_sizes: Vec<_> = splits.iter().map(|s| s.in_sample.height()).collect();
    assert!(is_sizes.windows(2).all(|w| w[0] < w[1]));
}
```

## Performance Considerations

1. **Parallel Fold Evaluation**: Use Rayon to evaluate folds in parallel
2. **Caching**: Cache indicator calculations across folds
3. **Memory**: Clear intermediate DataFrames to reduce memory usage
4. **Progress Tracking**: Report progress for each fold

## Common Issues

### Issue: OOS performance much worse than IS
**Solution**: This is normal and expected. Strategy is optimized on IS data. If gap is too large, increase regularization or use different fitness metrics.

### Issue: Inconsistent results across folds
**Solution**: Check data stationarity. Markets change over time. Consider using adaptive parameters or shorter windows.

### Issue: Not enough data for N folds
**Solution**: Reduce `n_folds` or use simpler validation method. Need minimum ~100 trades per fold for statistical significance.

## Next Steps

Proceed to **[14-ml-feature-engineering.md](./14-ml-feature-engineering.md)** to implement the ML pipeline for signal filtering and enhancement.
