# 04 - Indicators: Tier 1 (Must-Have)

## Goal
Implement the 10 must-have indicators that form the core of the indicator library.

## Prerequisites
- **[03-primitives.md](./03-primitives.md)** completed
- All 12 primitives implemented
- `cargo test primitives_test` passes

## What You'll Create
10 Tier 1 indicators across 4 files:
- **trend.rs**: SMA, EMA, MACD, Bollinger Bands
- **momentum.rs**: RSI, Stochastic, CCI
- **volatility.rs**: ATR, ADX
- **volume.rs**: OBV

## Tier 1 Indicator Overview

| Indicator | Mode | File | Complexity | Notes |
|-----------|------|------|------------|-------|
| SMA | Vectorized | trend.rs | Low | Delegates to primitive |
| EMA | Vectorized | trend.rs | Low | Delegates to primitive |
| RSI | Vectorized | momentum.rs | Medium | Wilder smoothing |
| MACD | Vectorized | trend.rs | Medium | EMA composition |
| BB | Vectorized | trend.rs | Medium | SMA + StdDev |
| ATR | Vectorized | volatility.rs | Medium | True Range + SMMA |
| Stochastic | Vectorized | momentum.rs | Medium | High/Low normalization |
| ADX | **Stateful** | volatility.rs | **High** | Complex state |
| OBV | Vectorized | volume.rs | Low | Cumulative volume |
| CCI | Vectorized | momentum.rs | Medium | Mean deviation |

**Note**: ADX is the only stateful indicator in Tier 1 due to its complex multi-step smoothing.

## Implementation Steps

### Step 1: Create Indicator Files

Create these files in `src/functions/indicators/`:
- `trend.rs`
- `momentum.rs`
- `volatility.rs`
- `volume.rs`

### Step 2: Implement Simple Indicators (SMA, EMA)

In `src/functions/indicators/trend.rs`:

```rust
use crate::functions::traits::*;
use crate::functions::primitives::MovingAverage;
use crate::types::*;
use crate::error::*;
use polars::prelude::*;

// SMA - Simple Moving Average
pub struct SMA;

impl Indicator for SMA {
    fn alias(&self) -> &'static str { "SMA" }
    fn ui_name(&self) -> &'static str { "Simple Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMA(_Symbol, _Period, {}, 0, MODE_SMA, PRICE_CLOSE, {})",
            args[1], // period
            "{}")    // shift placeholder
    }
}

impl VectorizedIndicator for SMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = args[0].as_series()?;
        let period = args[1].as_scalar()? as usize;

        // Delegate to MovingAverage primitive
        let ma = MovingAverage::sma();
        ma.execute(&[series, lit(period as i64)])
    }
}

// EMA - Exponential Moving Average
pub struct EMA;

impl Indicator for EMA {
    fn alias(&self) -> &'static str { "EMA" }
    fn ui_name(&self) -> &'static str { "Exponential Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMA(_Symbol, _Period, {}, 0, MODE_EMA, PRICE_CLOSE, {})",
            args[1], "{}")
    }
}

impl VectorizedIndicator for EMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = args[0].as_series()?;
        let period = args[1].as_scalar()? as usize;

        // Delegate to MovingAverage primitive
        let ma = MovingAverage::ema();
        ma.execute(&[series, lit(period as i64)])
    }
}
```

### Step 3: Implement RSI (Vectorized with Wilder Smoothing)

In `src/functions/indicators/momentum.rs`:

```rust
use crate::functions::traits::*;
use crate::types::*;
use crate::error::*;
use polars::prelude::*;

pub struct RSI;

impl Indicator for RSI {
    fn alias(&self) -> &'static str { "RSI" }
    fn ui_name(&self) -> &'static str { "Relative Strength Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iRSI(_Symbol, _Period, {}, PRICE_CLOSE, {})", args[1], "{}")
    }
}

impl VectorizedIndicator for RSI {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = args[0].as_series()?;
        let period = args[1].as_scalar()? as i32;

        // Step 1: Calculate price changes
        let delta = series.clone().diff(1, Default::default());

        // Step 2: Separate gains and losses
        let gains = when(delta.clone().gt(lit(0.0)))
            .then(delta.clone())
            .otherwise(lit(0.0));

        let losses = when(delta.clone().lt(lit(0.0)))
            .then(delta.abs())
            .otherwise(lit(0.0));

        // Step 3: Apply Wilder's smoothing (SMMA = EWM with alpha = 1/period)
        let alpha = 1.0 / period as f64;
        let avg_gains = gains.ewm_mean(EWMOptions {
            alpha,
            adjust: false,
            min_periods: period as usize,
            ..Default::default()
        });

        let avg_losses = losses.ewm_mean(EWMOptions {
            alpha,
            adjust: false,
            min_periods: period as usize,
            ..Default::default()
        });

        // Step 4: Calculate RS and RSI
        // RSI = 100 - (100 / (1 + RS))
        // Rewrite to avoid division issues: RSI = 100 * avg_gains / (avg_gains + avg_losses)
        let denominator = avg_gains.clone() + avg_losses.clone();
        let rsi = when(denominator.clone().gt(lit(0.0)))
            .then(lit(100.0) * avg_gains / denominator)
            .otherwise(lit(50.0)); // Default to midpoint if no data

        Ok(rsi)
    }
}
```

### Step 4: Implement MACD (Composed from EMAs)

Add to `src/functions/indicators/trend.rs`:

```rust
pub struct MACD;

impl Indicator for MACD {
    fn alias(&self) -> &'static str { "MACD" }
    fn ui_name(&self) -> &'static str { "Moving Average Convergence Divergence" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 4 }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,  // close
            DataType::Integer,        // fast period
            DataType::Integer,        // slow period
            DataType::Integer,        // signal period
        ]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMACD(_Symbol, _Period, {}, {}, {}, PRICE_CLOSE, MODE_MAIN, {})",
            args[1], args[2], args[3], "{}")
    }
}

impl VectorizedIndicator for MACD {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = args[0].as_series()?;
        let fast = args[1].as_scalar()? as usize;
        let slow = args[2].as_scalar()? as usize;
        let signal = args[3].as_scalar()? as usize;

        // MACD Line = EMA(fast) - EMA(slow)
        let ema_fast = series.clone().ewm_mean(EWMOptions {
            span: fast,
            min_periods: fast,
            ..Default::default()
        });

        let ema_slow = series.clone().ewm_mean(EWMOptions {
            span: slow,
            min_periods: slow,
            ..Default::default()
        });

        let macd_line = ema_fast - ema_slow;

        // Return MACD line (signal line and histogram can be calculated separately if needed)
        Ok(macd_line)
    }
}
```

### Step 5: Implement Bollinger Bands

Add to `src/functions/indicators/trend.rs`:

```rust
pub struct BollingerBands;

impl Indicator for BollingerBands {
    fn alias(&self) -> &'static str { "BB" }
    fn ui_name(&self) -> &'static str { "Bollinger Bands" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,  // close
            DataType::Integer,        // period
            DataType::Float,          // deviation
        ]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iBands(_Symbol, _Period, {}, {}, 0, PRICE_CLOSE, MODE_UPPER, {})",
            args[1], args[2], "{}")
    }
}

impl VectorizedIndicator for BollingerBands {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = args[0].as_series()?;
        let period = args[1].as_scalar()? as usize;
        let deviation = args[2].as_scalar()?;

        // Middle band = SMA
        let middle = series.clone().rolling_mean(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        // Standard deviation
        let std = series.clone().rolling_std(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        // Upper band = Middle + (deviation * std)
        // Return middle band (upper/lower can be calculated by caller)
        Ok(middle)
    }
}
```

### Step 6: Implement ATR (True Range with Wilder Smoothing)

Create `src/functions/indicators/volatility.rs`:

```rust
use crate::functions::traits::*;
use crate::types::*;
use crate::error::*;
use polars::prelude::*;

pub struct ATR;

impl Indicator for ATR {
    fn alias(&self) -> &'static str { "ATR" }
    fn ui_name(&self) -> &'static str { "Average True Range" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volatility }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 4 }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,  // high
            DataType::NumericSeries,  // low
            DataType::NumericSeries,  // close
            DataType::Integer,        // period
        ]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iATR(_Symbol, _Period, {}, {})", args[3], "{}")
    }
}

impl VectorizedIndicator for ATR {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let high = args[0].as_series()?;
        let low = args[1].as_series()?;
        let close = args[2].as_series()?;
        let period = args[3].as_scalar()? as usize;

        // Previous close
        let prev_close = close.clone().shift(lit(1));

        // Three components of True Range
        let hl = high.clone() - low.clone();
        let hc = (high.clone() - prev_close.clone()).abs();
        let lc = (low.clone() - prev_close).abs();

        // True Range = max(hl, hc, lc)
        let tr = hl.clone()
            .max(hc.clone())
            .max(lc.clone());

        // ATR = Wilder's smoothing of True Range
        let alpha = 1.0 / period as f64;
        let atr = tr.ewm_mean(EWMOptions {
            alpha,
            adjust: false,
            min_periods: period,
            ..Default::default()
        });

        Ok(atr)
    }
}
```

### Step 7: Implement ADX (Stateful - Complex)

Add to `src/functions/indicators/volatility.rs`:

```rust
use std::any::Any;
use std::collections::VecDeque;

pub struct ADX;

// State for ADX calculation
struct ADXState {
    period: usize,
    prev_high: f64,
    prev_low: f64,
    prev_close: f64,
    smoothed_pdm: f64,
    smoothed_ndm: f64,
    smoothed_tr: f64,
    dx_values: VecDeque<f64>,
    initialized: bool,
}

impl Indicator for ADX {
    fn alias(&self) -> &'static str { "ADX" }
    fn ui_name(&self) -> &'static str { "Average Directional Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 4 }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,  // high
            DataType::NumericSeries,  // low
            DataType::NumericSeries,  // close
            DataType::Integer,        // period
        ]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Stateful }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iADX(_Symbol, _Period, {}, PRICE_CLOSE, MODE_MAIN, {})",
            args[3], "{}")
    }
}

impl StatefulIndicator for ADX {
    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ADXState {
            period: 14,
            prev_high: 0.0,
            prev_low: 0.0,
            prev_close: 0.0,
            smoothed_pdm: 0.0,
            smoothed_ndm: 0.0,
            smoothed_tr: 0.0,
            dx_values: VecDeque::new(),
            initialized: false,
        })
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let adx_state = state.downcast_mut::<ADXState>()
            .ok_or_else(|| TradeBiasError::CalculationError("Invalid state type".to_string()))?;

        if args.len() != 4 {
            return Err(TradeBiasError::InvalidParameter(
                "ADX requires 4 args: high, low, close, period".to_string()
            ));
        }

        let high = args[0];
        let low = args[1];
        let close = args[2];
        let period = args[3] as usize;

        if !adx_state.initialized {
            adx_state.period = period;
            adx_state.prev_high = high;
            adx_state.prev_low = low;
            adx_state.prev_close = close;
            adx_state.initialized = true;
            return Ok(50.0); // Default value
        }

        // Calculate directional movements
        let pdm = (high - adx_state.prev_high).max(0.0);
        let ndm = (adx_state.prev_low - low).max(0.0);

        // Calculate true range
        let hl = high - low;
        let hc = (high - adx_state.prev_close).abs();
        let lc = (low - adx_state.prev_close).abs();
        let tr = hl.max(hc).max(lc);

        // Wilder's smoothing
        let alpha = 1.0 / period as f64;
        adx_state.smoothed_pdm = adx_state.smoothed_pdm * (1.0 - alpha) + pdm * alpha;
        adx_state.smoothed_ndm = adx_state.smoothed_ndm * (1.0 - alpha) + ndm * alpha;
        adx_state.smoothed_tr = adx_state.smoothed_tr * (1.0 - alpha) + tr * alpha;

        // Calculate +DI and -DI
        let pdi = if adx_state.smoothed_tr > 0.0 {
            100.0 * adx_state.smoothed_pdm / adx_state.smoothed_tr
        } else {
            0.0
        };

        let ndi = if adx_state.smoothed_tr > 0.0 {
            100.0 * adx_state.smoothed_ndm / adx_state.smoothed_tr
        } else {
            0.0
        };

        // Calculate DX
        let dx_sum = pdi + ndi;
        let dx = if dx_sum > 0.0 {
            100.0 * (pdi - ndi).abs() / dx_sum
        } else {
            0.0
        };

        // Store DX value
        adx_state.dx_values.push_back(dx);
        if adx_state.dx_values.len() > period {
            adx_state.dx_values.pop_front();
        }

        // Calculate ADX (average of DX)
        let adx = if adx_state.dx_values.len() == period {
            adx_state.dx_values.iter().sum::<f64>() / period as f64
        } else {
            50.0 // Default until we have enough values
        };

        // Update previous values
        adx_state.prev_high = high;
        adx_state.prev_low = low;
        adx_state.prev_close = close;

        Ok(adx)
    }
}
```

### Step 8: Implement Remaining Indicators

Create similar implementations for:
- **Stochastic** in `momentum.rs` (vectorized with high/low normalization)
- **CCI** in `momentum.rs` (vectorized with mean deviation)
- **OBV** in `volume.rs` (vectorized cumulative volume)

See the actual implementations in your source files for complete details.

### Step 9: Export All Indicators

Update `src/functions/indicators/mod.rs`:

```rust
pub mod trend;
pub mod momentum;
pub mod volatility;
pub mod volume;

pub use trend::*;
pub use momentum::*;
pub use volatility::*;
pub use volume::*;
```

## Verification

### Test Compilation

```bash
cargo check
```

### Test Individual Indicators

```bash
cargo test --test indicator_test
```

### Verify All 10 Indicators

```rust
#[test]
fn test_tier1_indicators() {
    let indicators: Vec<Box<dyn Indicator>> = vec![
        Box::new(SMA),
        Box::new(EMA),
        Box::new(RSI),
        Box::new(MACD),
        Box::new(BollingerBands),
        Box::new(ATR),
        Box::new(Stochastic),
        Box::new(ADX),
        Box::new(OBV),
        Box::new(CCI),
    ];

    assert_eq!(indicators.len(), 10);

    // Check that ADX is the only stateful one
    let stateful_count = indicators.iter()
        .filter(|i| i.calculation_mode() == CalculationMode::Stateful)
        .count();
    assert_eq!(stateful_count, 1);
}
```

## Common Issues

### Issue: ADX values incorrect
**Solution**: Make sure Wilder's smoothing is applied correctly with alpha = 1/period. Initialize smoothed values properly.

### Issue: RSI stuck at 50
**Solution**: Check that gains/losses are separated correctly and that EWM smoothing uses the right alpha parameter.

### Issue: MACD values too large
**Solution**: Verify that you're using the correct EMA periods (typically 12, 26, 9).

## Next Steps

Proceed to **[05-indicators-tier2.md](./05-indicators-tier2.md)** to implement the 20 common indicators.
