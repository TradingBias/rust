# 05 - Indicators: Tier 2 (Common)

## Goal
Implement 20 common indicators using the same patterns from Tier 1.

## Prerequisites
- **[04-indicators-tier1.md](./04-indicators-tier1.md)** completed
- All Tier 1 indicators working
- `cargo test indicator_test` passes

## Tier 2 Indicators (20 Total)

### Distribution Across Files

**momentum.rs** (7 indicators):
- WilliamsR - Vectorized
- MFI - Vectorized
- ROC - Vectorized
- DeMarker - Vectorized
- RVI - Vectorized
- Force - Vectorized
- TriX - Vectorized

**trend.rs** (6 indicators):
- DEMA - Vectorized
- TEMA - Vectorized
- Envelopes - Vectorized
- SAR - **Stateful** (complex conditional logic)
- Bulls - Vectorized
- Bears - Vectorized

**volatility.rs** (2 indicators):
- StdDev - Vectorized (already available as primitive)
- Chaikin - Vectorized

**volume.rs** (5 indicators):
- Volumes - Vectorized
- BWMFI - Vectorized
- AC - Vectorized
- AO - Vectorized
- Momentum (volume-based) - Vectorized

## Implementation Pattern

All Tier 2 indicators follow the same pattern as Tier 1:

1. **Implement `Indicator` trait** with metadata
2. **Implement either `VectorizedIndicator` or `StatefulIndicator`**
3. **Provide `generate_mql5()` method**

## Quick Implementation Guide

### Example: Williams %R (Vectorized)

```rust
pub struct WilliamsR;

impl Indicator for WilliamsR {
    fn alias(&self) -> &'static str { "WilliamsR" }
    fn ui_name(&self) -> &'static str { "Williams Percent Range" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((-100.0, 0.0)) }
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
        format!("iWPR(_Symbol, _Period, {}, {})", args[3], "{}")
    }
}

impl VectorizedIndicator for WilliamsR {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let high = args[0].as_series()?;
        let low = args[1].as_series()?;
        let close = args[2].as_series()?;
        let period = args[3].as_scalar()? as usize;

        // Highest high
        let hh = high.clone().rolling_max(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        // Lowest low
        let ll = low.clone().rolling_min(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        // %R = -100 * (HH - Close) / (HH - LL)
        let williams_r = lit(-100.0) * (hh.clone() - close) / (hh - ll);

        Ok(williams_r)
    }
}
```

### Example: SAR (Stateful - Complex)

```rust
pub struct SAR;

struct SARState {
    is_long: bool,
    sar: f64,
    ep: f64,  // Extreme point
    af: f64,  // Acceleration factor
    af_step: f64,
    af_max: f64,
}

impl StatefulIndicator for SAR {
    fn init_state(&self) -> Box<dyn Any> {
        Box::new(SARState {
            is_long: true,
            sar: 0.0,
            ep: 0.0,
            af: 0.02,
            af_step: 0.02,
            af_max: 0.2,
        })
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let sar_state = state.downcast_mut::<SARState>()
            .ok_or_else(|| TradeBiasError::CalculationError("Invalid state".to_string()))?;

        let high = args[0];
        let low = args[1];

        // Complex SAR calculation with acceleration factor logic
        // (Full implementation in actual source code)

        Ok(sar_state.sar)
    }
}
```

## Formulas Quick Reference

| Indicator | Formula | Key Operations |
|-----------|---------|----------------|
| Williams %R | -100 * (HH - Close) / (HH - LL) | rolling_max, rolling_min |
| MFI | Money Flow Index | typical_price * volume, ratio |
| ROC | (Close - Close[n]) / Close[n] * 100 | shift, division |
| DEMA | 2*EMA - EMA(EMA) | double smoothing |
| TEMA | 3*EMA - 3*EMA(EMA) + EMA(EMA(EMA)) | triple smoothing |
| SAR | Parabolic SAR with AF | stateful with reversals |
| Force | (Close - Close[1]) * Volume | momentum * volume |
| Envelopes | MA ± (MA * percent/100) | percentage bands |

## Verification

```bash
# Compile all indicators
cargo check

# Test Tier 2 indicators
cargo test tier2_indicators

# Verify count
cargo test --test indicator_count -- --nocapture
```

Expected output: 30 total indicators (10 Tier 1 + 20 Tier 2)

## Common Issues

**Issue**: SAR values incorrect
**Solution**: Ensure acceleration factor increments correctly and resets on trend reversal

**Issue**: MFI calculation wrong
**Solution**: Use typical price = (High + Low + Close) / 3, multiply by volume

## Next Steps

Proceed to **[06-registry-and-cache.md](./06-registry-and-cache.md)** to implement the function registry and caching system.
