# Critical Implementation Guide

**Status:** Active Implementation Guide
**Last Updated:** 2025-11-14
**Priority Level:** CRITICAL - Implement Immediately
**Total Estimated Effort:** 14-16 hours

## Overview

This document provides step-by-step implementation instructions for three critical features blocking core functionality. Follow the phases in order, as later phases depend on earlier ones.

---

# Phase 1: Comparison Operators (2-4 hours)

**Priority:** HIGHEST - Foundation for all other features
**File:** `src/functions/primitives.rs`

## Why This First?
Comparison operators are required by both indicators and strategies. Without them, you cannot:
- Create entry/exit conditions
- Compare indicator values to thresholds
- Detect crossovers
- Build meaningful strategies

## Implementation Steps

### Step 1.1: Define Comparison Enum (15 min)

Add a comparison operator enum to support all comparison types:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}
```

### Step 1.2: Implement Series-to-Series Comparisons (45 min)

Create a function to compare two Series:

```rust
pub fn compare_series(
    left: &Series,
    right: &Series,
    op: ComparisonOp,
) -> PolarsResult<Series> {
    // Validation
    if left.len() != right.len() {
        return Err(PolarsError::ShapeMismatch(
            format!("Series length mismatch: {} vs {}", left.len(), right.len()).into()
        ));
    }

    // Perform comparison based on operator
    match op {
        ComparisonOp::GreaterThan => left.gt(right),
        ComparisonOp::LessThan => left.lt(right),
        ComparisonOp::GreaterThanOrEqual => left.gt_eq(right),
        ComparisonOp::LessThanOrEqual => left.lt_eq(right),
        ComparisonOp::Equal => left.equal(right),
        ComparisonOp::NotEqual => left.not_equal(right),
    }
}
```

### Step 1.3: Implement Series-to-Scalar Comparisons (30 min)

Create functions for comparing Series to scalar values:

```rust
pub fn compare_to_scalar<T>(
    series: &Series,
    value: T,
    op: ComparisonOp,
) -> PolarsResult<Series>
where
    T: Into<AnyValue<'static>>,
{
    let scalar_series = Series::new("scalar", vec![value.into()]);
    let broadcast_scalar = scalar_series.new_from_index(0, series.len());

    compare_series(series, &broadcast_scalar, op)
}
```

### Step 1.4: Implement Convenience Functions (30 min)

Add user-friendly wrapper functions:

```rust
// Greater than
pub fn gt(left: &Series, right: &Series) -> PolarsResult<Series> {
    compare_series(left, right, ComparisonOp::GreaterThan)
}

pub fn gt_scalar<T>(series: &Series, value: T) -> PolarsResult<Series>
where
    T: Into<AnyValue<'static>>,
{
    compare_to_scalar(series, value, ComparisonOp::GreaterThan)
}

// Less than
pub fn lt(left: &Series, right: &Series) -> PolarsResult<Series> {
    compare_series(left, right, ComparisonOp::LessThan)
}

pub fn lt_scalar<T>(series: &Series, value: T) -> PolarsResult<Series>
where
    T: Into<AnyValue<'static>>,
{
    compare_to_scalar(series, value, ComparisonOp::LessThan)
}

// Implement gte, lte, eq, neq similarly...
```

### Step 1.5: Implement Cross Detection (60 min)

Add specialized crossover detection functions:

```rust
/// Detects when series1 crosses above series2
/// Returns true at the point of crossover
pub fn cross_above(series1: &Series, series2: &Series) -> PolarsResult<Series> {
    // Current: series1 > series2
    let current_above = series1.gt(series2)?;

    // Previous: series1[i-1] <= series2[i-1]
    let prev_series1 = series1.shift(1);
    let prev_series2 = series2.shift(1);
    let prev_below_or_equal = prev_series1.lt_eq(&prev_series2)?;

    // Cross above: current_above AND prev_below_or_equal
    current_above.bitand(&prev_below_or_equal)
}

/// Detects when series1 crosses below series2
pub fn cross_below(series1: &Series, series2: &Series) -> PolarsResult<Series> {
    // Current: series1 < series2
    let current_below = series1.lt(series2)?;

    // Previous: series1[i-1] >= series2[i-1]
    let prev_series1 = series1.shift(1);
    let prev_series2 = series2.shift(1);
    let prev_above_or_equal = prev_series1.gt_eq(&prev_series2)?;

    // Cross below: current_below AND prev_above_or_equal
    current_below.bitand(&prev_above_or_equal)
}
```

### Step 1.6: Write Tests (30 min)

Create `tests/comparison_operators.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_series_greater_than() {
        let s1 = Series::new("a", &[1.0, 2.0, 3.0, 4.0]);
        let s2 = Series::new("b", &[2.0, 2.0, 2.0, 2.0]);

        let result = gt(&s1, &s2).unwrap();
        let expected = Series::new("result", &[false, false, true, true]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_scalar_comparison() {
        let s = Series::new("rsi", &[30.0, 50.0, 70.0, 80.0]);

        let result = gt_scalar(&s, 70.0).unwrap();
        let expected = Series::new("result", &[false, false, false, true]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_cross_above() {
        let fast = Series::new("fast", &[1.0, 2.0, 3.0, 4.0, 3.0]);
        let slow = Series::new("slow", &[2.0, 2.0, 2.0, 2.0, 2.0]);

        let result = cross_above(&fast, &slow).unwrap();
        // Crosses above at index 2: 3.0 > 2.0 and prev 2.0 <= 2.0
        let expected = Series::new("result", &[false, false, true, false, false]);

        assert_eq!(result, expected);
    }
}
```

### Step 1.7: Verification

Run tests and verify:

```bash
cargo test comparison_operators
```

**Success Criteria:**
- [ ] All comparison operators work with Series-to-Series
- [ ] All comparison operators work with Series-to-Scalar
- [ ] Cross detection functions correctly identify crossovers
- [ ] All tests pass
- [ ] No performance regression

---

# Phase 2: Unrealized P&L Tracking (1-2 hours)

**Priority:** HIGH - Required for accurate metrics
**File:** `src/engines/evaluation/portfolio.rs`

## Why This Second?
Accurate portfolio valuation is essential for testing the indicators and strategies you'll implement in Phase 3.

## Implementation Steps

### Step 2.1: Analyze Current Portfolio Structure (15 min)

Read the current `portfolio.rs` implementation:

```bash
# Examine the Portfolio struct
# Look for: current positions, equity tracking, P&L calculation
```

Identify:
- How open positions are stored
- Current equity calculation
- Where realized P&L is tracked

### Step 2.2: Add Unrealized P&L Fields (15 min)

Add fields to track unrealized P&L in the Portfolio struct:

```rust
pub struct Portfolio {
    // Existing fields...
    pub realized_pnl: f64,

    // NEW: Add these fields
    pub unrealized_pnl: f64,
    pub total_pnl: f64,
    pub current_positions_value: f64,
}
```

### Step 2.3: Implement Unrealized P&L Calculation (30 min)

Add method to calculate unrealized P&L for open positions:

```rust
impl Portfolio {
    /// Calculate unrealized P&L for all open positions
    pub fn calculate_unrealized_pnl(&mut self, current_prices: &HashMap<String, f64>) {
        let mut total_unrealized = 0.0;
        let mut positions_value = 0.0;

        for (symbol, position) in &self.positions {
            if let Some(current_price) = current_prices.get(symbol) {
                // Calculate unrealized P&L
                let entry_value = position.quantity * position.entry_price;
                let current_value = position.quantity * current_price;
                let position_pnl = current_value - entry_value;

                total_unrealized += position_pnl;
                positions_value += current_value;
            }
        }

        self.unrealized_pnl = total_unrealized;
        self.current_positions_value = positions_value;
        self.total_pnl = self.realized_pnl + self.unrealized_pnl;
    }
}
```

### Step 2.4: Update Portfolio Value Calculation (20 min)

Modify the equity calculation to include unrealized P&L:

```rust
impl Portfolio {
    /// Get total portfolio value (cash + positions at current prices)
    pub fn total_value(&self) -> f64 {
        self.cash + self.current_positions_value
    }

    /// Get equity (initial capital + total P&L)
    pub fn equity(&self) -> f64 {
        self.initial_capital + self.total_pnl
    }
}
```

### Step 2.5: Update Drawdown Calculation (20 min)

Modify drawdown to use total portfolio value:

```rust
impl Portfolio {
    pub fn update_drawdown(&mut self) {
        let current_equity = self.equity();

        // Update peak equity
        if current_equity > self.peak_equity {
            self.peak_equity = current_equity;
        }

        // Calculate current drawdown
        if self.peak_equity > 0.0 {
            self.current_drawdown = (self.peak_equity - current_equity) / self.peak_equity;

            // Update max drawdown
            if self.current_drawdown > self.max_drawdown {
                self.max_drawdown = self.current_drawdown;
            }
        }
    }
}
```

### Step 2.6: Update Portfolio on Each Bar (10 min)

Ensure unrealized P&L is calculated on every price update:

```rust
// In the backtester's main loop
pub fn process_bar(&mut self, bar: &Bar) {
    // Update current prices
    self.current_prices.insert(bar.symbol.clone(), bar.close);

    // Calculate unrealized P&L with current prices
    self.portfolio.calculate_unrealized_pnl(&self.current_prices);

    // Update drawdown with current equity
    self.portfolio.update_drawdown();

    // ... rest of bar processing
}
```

### Step 2.7: Write Tests (20 min)

Add tests in `tests/portfolio_comprehensive.rs`:

```rust
#[test]
fn test_unrealized_pnl_long_position() {
    let mut portfolio = Portfolio::new(10000.0);

    // Buy 100 shares at $50
    portfolio.open_position("AAPL", 100.0, 50.0);

    // Price increases to $55
    let mut prices = HashMap::new();
    prices.insert("AAPL".to_string(), 55.0);
    portfolio.calculate_unrealized_pnl(&prices);

    // Unrealized P&L should be (55-50) * 100 = $500
    assert_eq!(portfolio.unrealized_pnl, 500.0);
    assert_eq!(portfolio.total_pnl, 500.0);
}

#[test]
fn test_portfolio_value_with_open_positions() {
    let mut portfolio = Portfolio::new(10000.0);

    // Buy 100 shares at $50 (costs $5000)
    portfolio.open_position("AAPL", 100.0, 50.0);
    // Cash remaining: $5000

    // Price increases to $60
    let mut prices = HashMap::new();
    prices.insert("AAPL".to_string(), 60.0);
    portfolio.calculate_unrealized_pnl(&prices);

    // Position value: 100 * $60 = $6000
    // Total value: $5000 cash + $6000 position = $11000
    assert_eq!(portfolio.total_value(), 11000.0);
}

#[test]
fn test_drawdown_with_unrealized_losses() {
    let mut portfolio = Portfolio::new(10000.0);
    portfolio.open_position("AAPL", 100.0, 100.0);

    let mut prices = HashMap::new();

    // Price drops to $80 (-20%)
    prices.insert("AAPL".to_string(), 80.0);
    portfolio.calculate_unrealized_pnl(&prices);
    portfolio.update_drawdown();

    // Unrealized loss: (80-100) * 100 = -$2000
    // Equity: $10000 - $2000 = $8000
    // Drawdown: 20%
    assert_eq!(portfolio.unrealized_pnl, -2000.0);
    assert_eq!(portfolio.current_drawdown, 0.20);
}
```

### Step 2.8: Verification

Run tests and verify functionality:

```bash
cargo test portfolio_comprehensive
cargo test backtester_comprehensive
```

**Success Criteria:**
- [ ] Unrealized P&L calculated correctly for long/short positions
- [ ] Portfolio value includes both cash and position values
- [ ] Drawdown accounts for unrealized losses
- [ ] All existing tests still pass
- [ ] New tests pass

---

# Phase 3: VectorizedIndicator Implementation (8-16 hours)

**Priority:** CRITICAL - Enables strategy functionality
**Files:** `src/functions/indicators/*`

This phase is broken into sub-phases based on indicator complexity.

## Phase 3A: Infrastructure & Simple Indicators (4-5 hours)

### Step 3A.1: Create VectorizedIndicator Trait (30 min)

Create `src/functions/indicators/mod.rs` or extend existing:

```rust
use polars::prelude::*;

/// Trait for vectorized technical indicators
pub trait VectorizedIndicator {
    /// Calculate the indicator on a Series
    fn calculate(&self, data: &Series) -> PolarsResult<Series>;

    /// Calculate on a DataFrame using specified column
    fn calculate_on_df(&self, df: &DataFrame, column: &str) -> PolarsResult<Series> {
        let series = df.column(column)?;
        self.calculate(series)
    }

    /// Get indicator name
    fn name(&self) -> String;

    /// Get minimum required data points
    fn min_periods(&self) -> usize;
}

/// Configuration for indicators with periods
#[derive(Debug, Clone)]
pub struct IndicatorConfig {
    pub period: usize,
    pub name_suffix: Option<String>,
}
```

### Step 3A.2: Implement Simple Moving Average (60 min)

Create or update `src/functions/indicators/moving_averages.rs`:

```rust
use super::{VectorizedIndicator, IndicatorConfig};
use polars::prelude::*;

/// Simple Moving Average
pub struct SMA {
    config: IndicatorConfig,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: None,
            },
        }
    }

    pub fn with_name(period: usize, name: impl Into<String>) -> Self {
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: Some(name.into()),
            },
        }
    }
}

impl VectorizedIndicator for SMA {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // Validate input
        if data.len() < self.config.period {
            return Err(PolarsError::ComputeError(
                format!(
                    "Insufficient data: {} rows, need {}",
                    data.len(),
                    self.config.period
                ).into()
            ));
        }

        // Calculate rolling mean using Polars
        let sma = data
            .rolling_mean(RollingOptionsImpl {
                window_size: self.config.period,
                min_periods: self.config.period,
                center: false,
                ..Default::default()
            })?;

        Ok(sma.with_name(&self.name()))
    }

    fn name(&self) -> String {
        if let Some(suffix) = &self.config.name_suffix {
            suffix.clone()
        } else {
            format!("sma_{}", self.config.period)
        }
    }

    fn min_periods(&self) -> usize {
        self.config.period
    }
}
```

### Step 3A.3: Implement Exponential Moving Average (90 min)

Add to `moving_averages.rs`:

```rust
/// Exponential Moving Average
pub struct EMA {
    config: IndicatorConfig,
    alpha: f64,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: None,
            },
            alpha,
        }
    }

    pub fn with_name(period: usize, name: impl Into<String>) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: Some(name.into()),
            },
            alpha,
        }
    }
}

impl VectorizedIndicator for EMA {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // Validate input
        if data.len() < self.config.period {
            return Err(PolarsError::ComputeError(
                format!(
                    "Insufficient data: {} rows, need {}",
                    data.len(),
                    self.config.period
                ).into()
            ));
        }

        // Use Polars ewm (exponentially weighted moving average)
        let ema = data
            .ewm_mean(EWMOptions {
                alpha: self.alpha,
                adjust: false,
                min_periods: self.config.period,
                ..Default::default()
            })?;

        Ok(ema.with_name(&self.name()))
    }

    fn name(&self) -> String {
        if let Some(suffix) = &self.config.name_suffix {
            suffix.clone()
        } else {
            format!("ema_{}", self.config.period)
        }
    }

    fn min_periods(&self) -> usize {
        self.config.period
    }
}
```

### Step 3A.4: Implement Rate of Change (45 min)

Create `src/functions/indicators/momentum.rs`:

```rust
use super::{VectorizedIndicator, IndicatorConfig};
use polars::prelude::*;

/// Rate of Change indicator
pub struct ROC {
    config: IndicatorConfig,
}

impl ROC {
    pub fn new(period: usize) -> Self {
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: None,
            },
        }
    }
}

impl VectorizedIndicator for ROC {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // ROC = ((current - previous) / previous) * 100
        let shifted = data.shift(self.config.period as i64);
        let change = (data - &shifted)?;
        let roc = (change / shifted)? * 100.0;

        Ok(roc.with_name(&self.name()))
    }

    fn name(&self) -> String {
        format!("roc_{}", self.config.period)
    }

    fn min_periods(&self) -> usize {
        self.config.period + 1
    }
}
```

### Step 3A.5: Write Tests for Simple Indicators (60 min)

Create `tests/indicator_verification.rs` or extend existing:

```rust
use tradebias::functions::indicators::*;
use polars::prelude::*;

#[test]
fn test_sma_calculation() {
    let data = Series::new("close", &[1.0, 2.0, 3.0, 4.0, 5.0]);
    let sma = SMA::new(3);

    let result = sma.calculate(&data).unwrap();

    // First two values should be null/NaN (insufficient data)
    // Third value: (1+2+3)/3 = 2.0
    // Fourth value: (2+3+4)/3 = 3.0
    // Fifth value: (3+4+5)/3 = 4.0

    let values: Vec<Option<f64>> = result.f64().unwrap().into_iter().collect();
    assert_eq!(values[2], Some(2.0));
    assert_eq!(values[3], Some(3.0));
    assert_eq!(values[4], Some(4.0));
}

#[test]
fn test_ema_calculation() {
    let data = Series::new("close", &[1.0, 2.0, 3.0, 4.0, 5.0]);
    let ema = EMA::new(3);

    let result = ema.calculate(&data).unwrap();

    // Verify EMA is calculated (values should differ from SMA)
    assert!(result.len() == data.len());
}

#[test]
fn test_roc_calculation() {
    let data = Series::new("close", &[100.0, 105.0, 110.0, 115.0]);
    let roc = ROC::new(1);

    let result = roc.calculate(&data).unwrap();

    // ROC[1] = ((105-100)/100) * 100 = 5.0
    let values: Vec<Option<f64>> = result.f64().unwrap().into_iter().collect();
    assert_eq!(values[1], Some(5.0));
}
```

### Step 3A.6: Verification

```bash
cargo test indicator_verification
```

**Success Criteria:**
- [ ] VectorizedIndicator trait compiles
- [ ] SMA, EMA, ROC implemented and tested
- [ ] Indicators work with Polars DataFrames
- [ ] All tests pass

---

## Phase 3B: Medium Complexity Indicators (4-6 hours)

### Step 3B.1: Implement RSI (90 min)

Add to `momentum.rs`:

```rust
/// Relative Strength Index
pub struct RSI {
    config: IndicatorConfig,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            config: IndicatorConfig {
                period,
                name_suffix: None,
            },
        }
    }
}

impl VectorizedIndicator for RSI {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // Calculate price changes
        let delta = data.diff(1, polars::prelude::NullBehavior::Drop);

        // Separate gains and losses
        let gains = delta.apply(|val| {
            if let Some(v) = val {
                if v > 0.0 { Some(v) } else { Some(0.0) }
            } else {
                None
            }
        });

        let losses = delta.apply(|val| {
            if let Some(v) = val {
                if v < 0.0 { Some(-v) } else { Some(0.0) }
            } else {
                None
            }
        });

        // Calculate average gains and losses using EMA
        let avg_gain = gains.ewm_mean(EWMOptions {
            alpha: 1.0 / self.config.period as f64,
            adjust: false,
            min_periods: self.config.period,
            ..Default::default()
        })?;

        let avg_loss = losses.ewm_mean(EWMOptions {
            alpha: 1.0 / self.config.period as f64,
            adjust: false,
            min_periods: self.config.period,
            ..Default::default()
        })?;

        // Calculate RS and RSI
        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Ok(rsi.with_name(&self.name()))
    }

    fn name(&self) -> String {
        format!("rsi_{}", self.config.period)
    }

    fn min_periods(&self) -> usize {
        self.config.period + 1
    }
}
```

### Step 3B.2: Implement MACD (120 min)

Create `src/functions/indicators/trend.rs`:

```rust
use super::{VectorizedIndicator, EMA};
use polars::prelude::*;

/// MACD (Moving Average Convergence Divergence)
pub struct MACD {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}

impl MACD {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
        }
    }

    pub fn standard() -> Self {
        Self::new(12, 26, 9)
    }

    /// Calculate MACD returning (macd_line, signal_line, histogram)
    pub fn calculate_full(&self, data: &Series) -> PolarsResult<(Series, Series, Series)> {
        // Calculate fast and slow EMAs
        let fast_ema = EMA::new(self.fast_period).calculate(data)?;
        let slow_ema = EMA::new(self.slow_period).calculate(data)?;

        // MACD line = fast EMA - slow EMA
        let macd_line = (&fast_ema - &slow_ema)?
            .with_name("macd_line");

        // Signal line = EMA of MACD line
        let signal_line = EMA::new(self.signal_period)
            .calculate(&macd_line)?
            .with_name("signal_line");

        // Histogram = MACD line - signal line
        let histogram = (&macd_line - &signal_line)?
            .with_name("macd_histogram");

        Ok((macd_line, signal_line, histogram))
    }
}

impl VectorizedIndicator for MACD {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // Return MACD line by default
        let (macd_line, _, _) = self.calculate_full(data)?;
        Ok(macd_line)
    }

    fn name(&self) -> String {
        format!("macd_{}_{}", self.fast_period, self.slow_period)
    }

    fn min_periods(&self) -> usize {
        self.slow_period
    }
}
```

### Step 3B.3: Implement Bollinger Bands (90 min)

Add to `trend.rs`:

```rust
/// Bollinger Bands
pub struct BollingerBands {
    period: usize,
    num_std: f64,
}

impl BollingerBands {
    pub fn new(period: usize, num_std: f64) -> Self {
        Self { period, num_std }
    }

    pub fn standard() -> Self {
        Self::new(20, 2.0)
    }

    /// Calculate Bollinger Bands returning (upper, middle, lower)
    pub fn calculate_full(&self, data: &Series) -> PolarsResult<(Series, Series, Series)> {
        // Middle band = SMA
        let middle = data.rolling_mean(RollingOptionsImpl {
            window_size: self.period,
            min_periods: self.period,
            center: false,
            ..Default::default()
        })?;

        // Calculate standard deviation
        let std_dev = data.rolling_std(RollingOptionsImpl {
            window_size: self.period,
            min_periods: self.period,
            center: false,
            ..Default::default()
        })?;

        // Upper band = middle + (std_dev * num_std)
        let upper = (&middle + &std_dev * self.num_std)?
            .with_name("bb_upper");

        // Lower band = middle - (std_dev * num_std)
        let lower = (&middle - &std_dev * self.num_std)?
            .with_name("bb_lower");

        let middle = middle.with_name("bb_middle");

        Ok((upper, middle, lower))
    }
}

impl VectorizedIndicator for BollingerBands {
    fn calculate(&self, data: &Series) -> PolarsResult<Series> {
        // Return middle band by default
        let (_, middle, _) = self.calculate_full(data)?;
        Ok(middle)
    }

    fn name(&self) -> String {
        format!("bb_{}", self.period)
    }

    fn min_periods(&self) -> usize {
        self.period
    }
}
```

### Step 3B.4: Write Tests (60 min)

Add to `tests/indicator_verification.rs`:

```rust
#[test]
fn test_rsi_bounds() {
    // RSI should always be between 0 and 100
    let data = Series::new("close", &[
        100.0, 105.0, 103.0, 108.0, 110.0,
        107.0, 112.0, 115.0, 113.0, 118.0,
        120.0, 122.0, 119.0, 125.0, 123.0,
    ]);

    let rsi = RSI::new(14);
    let result = rsi.calculate(&data).unwrap();

    // Check all non-null values are in valid range
    for val in result.f64().unwrap().into_iter().flatten() {
        assert!(val >= 0.0 && val <= 100.0);
    }
}

#[test]
fn test_macd_components() {
    let data = Series::new("close", &[
        100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
        108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
        116.0, 118.0, 120.0, 119.0, 121.0, 123.0, 125.0,
        124.0, 126.0, 128.0, 127.0, 129.0, 131.0, 130.0,
    ]);

    let macd = MACD::standard();
    let (macd_line, signal, histogram) = macd.calculate_full(&data).unwrap();

    // All series should have same length
    assert_eq!(macd_line.len(), data.len());
    assert_eq!(signal.len(), data.len());
    assert_eq!(histogram.len(), data.len());
}

#[test]
fn test_bollinger_bands_relationship() {
    let data = Series::new("close", &[
        100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
        108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
        116.0, 118.0, 120.0, 119.0, 121.0, 123.0, 125.0,
    ]);

    let bb = BollingerBands::standard();
    let (upper, middle, lower) = bb.calculate_full(&data).unwrap();

    // Upper should be >= middle >= lower (ignoring nulls)
    let upper_vals: Vec<Option<f64>> = upper.f64().unwrap().into_iter().collect();
    let middle_vals: Vec<Option<f64>> = middle.f64().unwrap().into_iter().collect();
    let lower_vals: Vec<Option<f64>> = lower.f64().unwrap().into_iter().collect();

    for i in 0..data.len() {
        if let (Some(u), Some(m), Some(l)) = (upper_vals[i], middle_vals[i], lower_vals[i]) {
            assert!(u >= m);
            assert!(m >= l);
        }
    }
}
```

### Step 3B.5: Verification

```bash
cargo test indicator_verification
```

**Success Criteria:**
- [ ] RSI values are always between 0-100
- [ ] MACD returns all three components
- [ ] Bollinger Bands upper >= middle >= lower
- [ ] All tests pass

---

## Phase 3C: Complex Indicators (Optional - 6-8 hours)

This phase covers advanced indicators. Implement these only if needed for specific strategies.

### Indicators to Implement:
1. **Stochastic Oscillator** (2-3 hours)
2. **Average Directional Index (ADX)** (2-3 hours)
3. **Ichimoku Cloud** (2-3 hours)

### Implementation Pattern:
Follow the same pattern as Phase 3B:
1. Create struct with configuration
2. Implement VectorizedIndicator trait
3. Add helper methods for multiple outputs
4. Write comprehensive tests
5. Verify against known values

---

# Integration & Testing

## Final Integration Test

Create `tests/strategy_integration.rs`:

```rust
use tradebias::functions::indicators::*;
use tradebias::functions::primitives::*;
use polars::prelude::*;

#[test]
fn test_ma_crossover_strategy() {
    // Create sample OHLCV data
    let df = df! {
        "timestamp" => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "close" => &[100.0, 102.0, 104.0, 103.0, 105.0,
                     107.0, 109.0, 108.0, 110.0, 112.0],
    }.unwrap();

    let close = df.column("close").unwrap();

    // Calculate indicators
    let fast_ma = SMA::new(3).calculate(close).unwrap();
    let slow_ma = SMA::new(5).calculate(close).unwrap();

    // Generate signals using comparison operators
    let buy_signal = cross_above(&fast_ma, &slow_ma).unwrap();
    let sell_signal = cross_below(&fast_ma, &slow_ma).unwrap();

    // Verify signals are boolean series
    assert_eq!(buy_signal.dtype(), &DataType::Boolean);
    assert_eq!(sell_signal.dtype(), &DataType::Boolean);

    // Add signals to dataframe
    let df = df
        .lazy()
        .with_column(fast_ma.lit().alias("fast_ma"))
        .with_column(slow_ma.lit().alias("slow_ma"))
        .with_column(buy_signal.lit().alias("buy_signal"))
        .with_column(sell_signal.lit().alias("sell_signal"))
        .collect()
        .unwrap();

    println!("{}", df);
}

#[test]
fn test_rsi_threshold_strategy() {
    let df = df! {
        "close" => &[
            100.0, 105.0, 103.0, 108.0, 110.0,
            107.0, 112.0, 115.0, 113.0, 118.0,
            120.0, 122.0, 119.0, 125.0, 123.0,
        ],
    }.unwrap();

    let close = df.column("close").unwrap();

    // Calculate RSI
    let rsi = RSI::new(14).calculate(close).unwrap();

    // Generate signals: buy when RSI < 30, sell when RSI > 70
    let oversold = lt_scalar(&rsi, 30.0).unwrap();
    let overbought = gt_scalar(&rsi, 70.0).unwrap();

    // Verify signals work
    assert_eq!(oversold.dtype(), &DataType::Boolean);
    assert_eq!(overbought.dtype(), &DataType::Boolean);
}
```

## Run Full Test Suite

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test comparison_operators
cargo test portfolio_comprehensive
cargo test indicator_verification
cargo test strategy_integration

# Run with output
cargo test -- --nocapture
```

---

# Completion Checklist

## Phase 1: Comparison Operators
- [ ] ComparisonOp enum defined
- [ ] Series-to-Series comparison functions
- [ ] Series-to-Scalar comparison functions
- [ ] Convenience functions (gt, lt, gte, lte, eq, neq)
- [ ] Cross detection functions (cross_above, cross_below)
- [ ] All tests pass
- [ ] Documentation added

## Phase 2: Unrealized P&L
- [ ] Unrealized P&L fields added to Portfolio
- [ ] calculate_unrealized_pnl() method implemented
- [ ] total_value() includes position values
- [ ] equity() includes total P&L
- [ ] Drawdown calculation updated
- [ ] Backtester calls calculate_unrealized_pnl() each bar
- [ ] All tests pass

## Phase 3A: Simple Indicators
- [ ] VectorizedIndicator trait defined
- [ ] SMA implemented and tested
- [ ] EMA implemented and tested
- [ ] ROC implemented and tested
- [ ] Integration test passes

## Phase 3B: Medium Indicators
- [ ] RSI implemented and tested
- [ ] MACD implemented with all components
- [ ] Bollinger Bands implemented with all bands
- [ ] All indicator tests pass

## Phase 3C: Complex Indicators (Optional)
- [ ] Stochastic Oscillator
- [ ] ADX
- [ ] Ichimoku Cloud

## Final Integration
- [ ] MA crossover strategy test passes
- [ ] RSI threshold strategy test passes
- [ ] Full test suite passes
- [ ] Performance benchmarks acceptable
- [ ] Documentation complete

---

# Success Metrics

Upon completion, you should be able to:

1. **Create Strategy Conditions:**
   ```rust
   let buy = cross_above(&fast_ma, &slow_ma)?;
   let sell = gt_scalar(&rsi, 70.0)?;
   ```

2. **Accurate Portfolio Tracking:**
   ```rust
   portfolio.calculate_unrealized_pnl(&prices);
   let total_value = portfolio.total_value(); // Includes open positions
   let drawdown = portfolio.current_drawdown; // Accounts for unrealized losses
   ```

3. **Use Indicators in Strategies:**
   ```rust
   let rsi = RSI::new(14).calculate(&close)?;
   let macd = MACD::standard().calculate(&close)?;
   let bb = BollingerBands::standard().calculate_full(&close)?;
   ```

---

# Troubleshooting

## Common Issues

### Polars API Changes
If rolling window functions don't work as expected, check:
- Current Polars version: `cargo tree | grep polars`
- Polars documentation for correct API

### Performance Issues
If indicators are slow:
- Use `.lazy()` evaluation where possible
- Batch indicator calculations
- Profile with `cargo flamegraph`

### Test Failures
If integration tests fail:
- Verify Phase 1 comparison operators work first
- Check that Phase 2 portfolio updates are called
- Ensure indicators return correct data types

---

**Total Implementation Time:** 14-16 hours
**Can be parallelized:** No (phases depend on each other)
**Priority:** CRITICAL - Blocks all strategy functionality
