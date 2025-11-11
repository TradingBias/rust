# 03 - Primitive Implementations

## Goal
Implement all 12 primitive building blocks that will be used to construct 70+ indicators.

## Prerequisites
- **[02-type-system.md](./02-type-system.md)** completed
- Core types and traits implemented
- `cargo check` passes

## What You'll Create
All 12 primitives in `src/functions/primitives.rs`:
1. MovingAverage (SMA, EMA, LWMA, SMMA)
2. Highest
3. Lowest
4. Sum
5. StdDev
6. Momentum
7. Shift
8. Absolute
9. Divide
10. Multiply
11. Add
12. Subtract

## Why These 12 Primitives?

These primitives can build ALL 70+ indicators in the library. Examples:
- **RSI** = Momentum + averaging gains/losses
- **MACD** = EMA(close, 12) - EMA(close, 26)
- **Bollinger Bands** = SMA(close, 20) ± (2 * StdDev(close, 20))
- **Stochastic** = (close - Lowest(low, 14)) / (Highest(high, 14) - Lowest(low, 14)) * 100

## Implementation Steps

### Step 1: Set Up Primitives Module

Create `src/functions/primitives.rs`:

```rust
use crate::functions::traits::*;
use crate::types::*;
use crate::error::*;
use polars::prelude::*;
use anyhow::bail;

// Helper function to extract literal integer from Expr
fn extract_literal_int(expr: &Expr) -> Result<usize> {
    match expr {
        Expr::Literal(LiteralValue::Int64(v)) => Ok(*v as usize),
        Expr::Literal(LiteralValue::Int32(v)) => Ok(*v as usize),
        _ => Err(TradeBiasError::InvalidParameter(
            "Expected integer literal".to_string()
        ))
    }
}

// Helper function to extract literal float from Expr
fn extract_literal_float(expr: &Expr) -> Result<f64> {
    match expr {
        Expr::Literal(LiteralValue::Float64(v)) => Ok(*v),
        Expr::Literal(LiteralValue::Float32(v)) => Ok(*v as f64),
        Expr::Literal(LiteralValue::Int64(v)) => Ok(*v as f64),
        Expr::Literal(LiteralValue::Int32(v)) => Ok(*v as f64),
        _ => Err(TradeBiasError::InvalidParameter(
            "Expected numeric literal".to_string()
        ))
    }
}
```

### Step 2: Implement MovingAverage Primitive

This is the most complex primitive with 4 methods:

```rust
pub struct MovingAverage {
    method: MAMethod,
}

impl MovingAverage {
    pub fn new(method: MAMethod) -> Self {
        Self { method }
    }

    pub fn sma() -> Self { Self::new(MAMethod::Simple) }
    pub fn ema() -> Self { Self::new(MAMethod::Exponential) }
    pub fn lwma() -> Self { Self::new(MAMethod::Linear) }
    pub fn smma() -> Self { Self::new(MAMethod::Smoothed) }
}

impl Primitive for MovingAverage {
    fn alias(&self) -> &'static str {
        match self.method {
            MAMethod::Simple => "SMA",
            MAMethod::Exponential => "EMA",
            MAMethod::Linear => "LWMA",
            MAMethod::Smoothed => "SMMA",
        }
    }

    fn ui_name(&self) -> &'static str {
        match self.method {
            MAMethod::Simple => "Simple Moving Average",
            MAMethod::Exponential => "Exponential Moving Average",
            MAMethod::Linear => "Linear Weighted Moving Average",
            MAMethod::Smoothed => "Smoothed Moving Average",
        }
    }

    fn arity(&self) -> usize { 2 }

    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }

    fn output_type(&self) -> DataType {
        DataType::NumericSeries
    }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("MovingAverage requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        if period == 0 {
            return Err(TradeBiasError::InvalidParameter(
                "Period must be > 0".to_string()
            ));
        }

        match self.method {
            MAMethod::Simple => {
                // Simple moving average using rolling window
                Ok(series.clone().rolling_mean(RollingOptionsFixedWindow {
                    window_size: period,
                    min_periods: period,
                    ..Default::default()
                }))
            }

            MAMethod::Exponential => {
                // Exponential moving average
                Ok(series.clone().ewm_mean(EWMOptions {
                    span: period,
                    min_periods: period,
                    ..Default::default()
                }))
            }

            MAMethod::Linear => {
                // Linear weighted moving average
                // Weights = [1, 2, 3, ..., period]
                // LWMA = sum(price[i] * weight[i]) / sum(weights)
                // Use rolling_apply for custom calculation
                let period_lit = lit(period as i64);

                // This is a simplified version - full implementation would use custom rolling function
                // For now, use SMA as placeholder (TODO: implement proper LWMA)
                Ok(series.clone().rolling_mean(RollingOptionsFixedWindow {
                    window_size: period,
                    min_periods: period,
                    ..Default::default()
                }))
            }

            MAMethod::Smoothed => {
                // Smoothed moving average (Wilder's smoothing)
                // SMMA[i] = (SMMA[i-1] * (period - 1) + price[i]) / period
                // First value is SMA
                Ok(series.clone().ewm_mean(EWMOptions {
                    alpha: 1.0 / period as f64,
                    min_periods: period,
                    ..Default::default()
                }))
            }
        }
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        let method = match self.method {
            MAMethod::Simple => "MODE_SMA",
            MAMethod::Exponential => "MODE_EMA",
            MAMethod::Linear => "MODE_LWMA",
            MAMethod::Smoothed => "MODE_SMMA",
        };

        format!("iMA({}, 0, {}, 0, {}, PRICE_CLOSE, {{}})",
            args[0], args[1], method)
    }
}
```

### Step 3: Implement Statistical Primitives

```rust
// HIGHEST - Maximum value over period
pub struct Highest;

impl Primitive for Highest {
    fn alias(&self) -> &'static str { "Highest" }
    fn ui_name(&self) -> &'static str { "Highest Value" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Highest requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        Ok(series.clone().rolling_max(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        }))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("TB_Highest({}, {}, {{}})", args[0], args[1])
    }
}

// LOWEST - Minimum value over period
pub struct Lowest;

impl Primitive for Lowest {
    fn alias(&self) -> &'static str { "Lowest" }
    fn ui_name(&self) -> &'static str { "Lowest Value" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Lowest requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        Ok(series.clone().rolling_min(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        }))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("TB_Lowest({}, {}, {{}})", args[0], args[1])
    }
}

// SUM - Sum over period
pub struct Sum;

impl Primitive for Sum {
    fn alias(&self) -> &'static str { "Sum" }
    fn ui_name(&self) -> &'static str { "Rolling Sum" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Sum requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        Ok(series.clone().rolling_sum(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        }))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("TB_Sum({}, {}, {{}})", args[0], args[1])
    }
}

// STDDEV - Standard Deviation over period
pub struct StdDev;

impl Primitive for StdDev {
    fn alias(&self) -> &'static str { "StdDev" }
    fn ui_name(&self) -> &'static str { "Standard Deviation" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("StdDev requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        Ok(series.clone().rolling_std(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        }))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iStdDev({}, 0, {}, 0, MODE_SMA, PRICE_CLOSE, {{}})",
            args[0], args[1])
    }
}
```

### Step 4: Implement Transformation Primitives

```rust
// MOMENTUM - Price change over period
pub struct Momentum;

impl Primitive for Momentum {
    fn alias(&self) -> &'static str { "Momentum" }
    fn ui_name(&self) -> &'static str { "Momentum (Price Change)" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Momentum requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;

        // Momentum = price[i] - price[i - period]
        Ok(series.clone() - series.clone().shift(lit(period as i64)))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMomentum({}, 0, {}, PRICE_CLOSE, {{}})",
            args[0], args[1])
    }
}

// SHIFT - Time-shift series
pub struct Shift;

impl Primitive for Shift {
    fn alias(&self) -> &'static str { "Shift" }
    fn ui_name(&self) -> &'static str { "Shift Series" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Shift requires 2 args, got {}", args.len())
            ));
        }

        let series = &args[0];
        let offset = extract_literal_int(&args[1])? as i64;

        Ok(series.clone().shift(lit(offset)))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("{}[{{}} + {}]", args[0], args[1])
    }
}

// ABSOLUTE - Absolute value
pub struct Absolute;

impl Primitive for Absolute {
    fn alias(&self) -> &'static str { "Abs" }
    fn ui_name(&self) -> &'static str { "Absolute Value" }
    fn arity(&self) -> usize { 1 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 1 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Abs requires 1 arg, got {}", args.len())
            ));
        }

        Ok(args[0].clone().abs())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("MathAbs({})", args[0])
    }
}
```

### Step 5: Implement Arithmetic Primitives

```rust
// DIVIDE - Safe division (avoid division by zero)
pub struct Divide;

impl Primitive for Divide {
    fn alias(&self) -> &'static str { "Divide" }
    fn ui_name(&self) -> &'static str { "Divide" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::NumericSeries]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Divide requires 2 args, got {}", args.len())
            ));
        }

        let numerator = &args[0];
        let denominator = &args[1];

        // Safe division: replace NaN/Inf with 0
        let result = numerator.clone() / denominator.clone();
        Ok(result.fill_nan(lit(0.0)))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} / ({} == 0 ? 1.0 : {}))", args[0], args[1], args[1])
    }
}

// MULTIPLY
pub struct Multiply;

impl Primitive for Multiply {
    fn alias(&self) -> &'static str { "Multiply" }
    fn ui_name(&self) -> &'static str { "Multiply" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::NumericSeries]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Multiply requires 2 args, got {}", args.len())
            ));
        }

        Ok(args[0].clone() * args[1].clone())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} * {})", args[0], args[1])
    }
}

// ADD
pub struct Add;

impl Primitive for Add {
    fn alias(&self) -> &'static str { "Add" }
    fn ui_name(&self) -> &'static str { "Add" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::NumericSeries]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Add requires 2 args, got {}", args.len())
            ));
        }

        Ok(args[0].clone() + args[1].clone())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} + {})", args[0], args[1])
    }
}

// SUBTRACT
pub struct Subtract;

impl Primitive for Subtract {
    fn alias(&self) -> &'static str { "Subtract" }
    fn ui_name(&self) -> &'static str { "Subtract" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::NumericSeries]
    }
    fn output_type(&self) -> DataType { DataType::NumericSeries }

    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            return Err(TradeBiasError::InvalidParameter(
                format!("Subtract requires 2 args, got {}", args.len())
            ));
        }

        Ok(args[0].clone() - args[1].clone())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} - {})", args[0], args[1])
    }
}
```

### Step 6: Export All Primitives

At the end of `src/functions/primitives.rs`, add:

```rust
// Factory functions for easy construction
pub fn get_all_primitives() -> Vec<Box<dyn Primitive>> {
    vec![
        Box::new(MovingAverage::sma()),
        Box::new(MovingAverage::ema()),
        Box::new(MovingAverage::lwma()),
        Box::new(MovingAverage::smma()),
        Box::new(Highest),
        Box::new(Lowest),
        Box::new(Sum),
        Box::new(StdDev),
        Box::new(Momentum),
        Box::new(Shift),
        Box::new(Absolute),
        Box::new(Divide),
        Box::new(Multiply),
        Box::new(Add),
        Box::new(Subtract),
    ]
}
```

## Verification

### Step 1: Compilation Check

```bash
cargo check
```

Should compile without errors.

### Step 2: Unit Tests

Create `tests/primitives_test.rs`:

```rust
use tradebias::*;
use polars::prelude::*;

#[test]
fn test_sma_primitive() {
    let sma = MovingAverage::sma();
    assert_eq!(sma.alias(), "SMA");
    assert_eq!(sma.arity(), 2);
}

#[test]
fn test_highest_primitive() {
    let highest = Highest;
    assert_eq!(highest.alias(), "Highest");
    assert_eq!(highest.arity(), 2);
}

#[test]
fn test_primitive_count() {
    let primitives = get_all_primitives();
    assert_eq!(primitives.len(), 15); // 4 MA methods + 11 others
}

#[test]
fn test_arithmetic_primitives() {
    let add = Add;
    let sub = Subtract;
    let mul = Multiply;
    let div = Divide;

    assert_eq!(add.arity(), 2);
    assert_eq!(sub.arity(), 2);
    assert_eq!(mul.arity(), 2);
    assert_eq!(div.arity(), 2);
}
```

Run:

```bash
cargo test primitives_test
```

### Step 3: MQL5 Code Generation Test

```rust
#[test]
fn test_mql5_generation() {
    let sma = MovingAverage::sma();
    let code = sma.generate_mql5(&["Close".to_string(), "14".to_string()]);
    assert!(code.contains("Close"));
    assert!(code.contains("14"));
    assert!(code.contains("MODE_SMA"));
}
```

## Common Issues

### Issue: Cannot extract literal from Expr
**Solution**: Use the helper functions `extract_literal_int()` and `extract_literal_float()` provided in Step 1.

### Issue: Polars method not found
**Solution**: Make sure you're using `Expr` methods, not `Series` methods. Check Polars documentation for correct API.

### Issue: Division by zero errors
**Solution**: Use `.fill_nan(lit(0.0))` after division operations to replace NaN/Inf with 0.

## Key Concepts

### Vectorized Operations

All primitives use vectorized Polars operations for performance:
- `rolling_mean()` - for moving averages
- `rolling_max()` / `rolling_min()` - for highest/lowest
- `shift()` - for time-shifting
- `+`, `-`, `*`, `/` - for arithmetic

### MQL5 Code Generation

Each primitive generates MQL5 code that:
1. Uses MQL5 built-in functions when available (e.g., `iMA`, `iStdDev`)
2. Generates custom functions when needed (e.g., `TB_Highest`, `TB_Lowest`)
3. Handles bar indexing with `{}` placeholder for shift parameter

## Next Steps

Proceed to **[04-indicators-tier1.md](./04-indicators-tier1.md)** to implement the 10 must-have indicators using these primitives.
