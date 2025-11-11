use std::any::Any;
use anyhow::{Result, bail};
use polars::lazy::dsl::{self, col, cum_sum, when}; // Corrected import for cum_sum and when
use polars::prelude::EWMOptions; // Added EWMOptions import
use crate::functions::traits::{Indicator, IndicatorArg, VectorizedIndicator}; // Added VectorizedIndicator
use crate::types::{DataType, ScaleType};
use std::collections::VecDeque; // Added VecDeque import

// --- OBV (On-Balance Volume) ---
pub struct OBV;

impl OBV {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indicator for OBV {
    fn alias(&self) -> &'static str { "OBV" }
    fn ui_name(&self) -> &'static str { "On-Balance Volume" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // close, volume
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::NumericSeries, // volume
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        "iOBV(_Symbol, _Period)".to_string()
    }
}

impl crate::functions::traits::VectorizedIndicator for OBV {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("OBV: first arg must be close series"),
        };
        let volume = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("OBV: second arg must be volume series"),
        };

        let prev_close = close.shift(1);
        let signed_volume = when(close.clone().gt(prev_close.clone()))
            .then(volume.clone())
            .when(close.lt(prev_close))
            .then(-volume)
            .otherwise(dsl::lit(0.0));

        Ok(cum_sum(signed_volume, false))
    }
}

// --- MFI (Money Flow Index) ---
pub struct MFI {
    pub period: usize,
}

impl MFI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for MFI {
    fn alias(&self) -> &'static str { "MFI" }
    fn ui_name(&self) -> &'static str { "Money Flow Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 5 } // high, low, close, volume, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::NumericSeries, // volume
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMFI(_Symbol, _Period, {})", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for MFI {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MFI: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MFI: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MFI: third arg must be close series"),
        };
        let volume = match &args[3] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MFI: fourth arg must be volume series"),
        };

        let typical_price = (high + low + close.clone()) / dsl::lit(3.0);
        let prev_typical_price = typical_price.shift(1);

        let raw_money_flow = typical_price * volume;

        let positive_money_flow = when(raw_money_flow.clone().gt(prev_typical_price.clone()))
            .then(raw_money_flow.clone())
            .otherwise(dsl::lit(0.0));

        let negative_money_flow = when(raw_money_flow.clone().lt(prev_typical_price))
            .then(raw_money_flow)
            .otherwise(dsl::lit(0.0));

        let positive_mf_sum = positive_money_flow.rolling_sum(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        let negative_mf_sum = negative_money_flow.rolling_sum(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        let money_ratio = positive_mf_sum.clone() / negative_mf_sum;
        let money_flow_index = dsl::lit(100.0) - (dsl::lit(100.0) / (dsl::lit(1.0) + money_ratio));

        Ok(money_flow_index)
    }
}
// --- Force Index ---
pub struct Force {
    pub period: usize,
}

impl Force {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for Force {
    fn alias(&self) -> &'static str { "Force" }
    fn ui_name(&self) -> &'static str { "Force Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 } // close, volume, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::NumericSeries, // volume
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iForce(_Symbol, _Period, {}, MODE_EMA, PRICE_CLOSE)", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for Force {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Force: first arg must be close series"),
        };
        let volume = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Force: second arg must be volume series"),
        };

        let force = (close.clone() - close.shift(1)) * volume;

        Ok(force.ewm_mean(EWMOptions {
            alpha: 2.0 / (self.period as f64 + 1.0),
            adjust: false,
            min_periods: self.period,
            ..Default::default()
        }))
    }
}
// --- Volumes ---
pub struct Volumes;

impl Volumes {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indicator for Volumes {
    fn alias(&self) -> &'static str { "Volumes" }
    fn ui_name(&self) -> &'static str { "Volumes" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 1 } // volume
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        "iVolumes(_Symbol, _Period)".to_string()
    }
}

impl crate::functions::traits::VectorizedIndicator for Volumes {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let volume = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Volumes: first arg must be volume series"),
        };
        Ok(volume)
    }
}
// --- Chaikin Oscillator ---
pub struct Chaikin {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Chaikin {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self { fast_period, slow_period }
    }
}

impl Indicator for Chaikin {
    fn alias(&self) -> &'static str { "Chaikin" }
    fn ui_name(&self) -> &'static str { "Chaikin Oscillator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 5 } // high, low, close, volume, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::NumericSeries, // volume
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iAD(_Symbol, _Period)")
    }
}

impl crate::functions::traits::VectorizedIndicator for Chaikin {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Chaikin: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Chaikin: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Chaikin: third arg must be close series"),
        };
        let volume = match &args[3] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Chaikin: fourth arg must be volume series"),
        };

        let money_flow_multiplier = ((close.clone() - low) - (high.clone() - close.clone())) / (high - low.clone());
        let money_flow_volume = money_flow_multiplier * volume;
        let adl = cum_sum(money_flow_volume, false);

        let ema_fast = adl.clone().ewm_mean(EWMOptions {
            alpha: 2.0 / (self.fast_period as f64 + 1.0),
            adjust: false,
            min_periods: self.fast_period,
            ..Default::default()
        });

        let ema_slow = adl.ewm_mean(EWMOptions {
            alpha: 2.0 / (self.slow_period as f64 + 1.0),
            adjust: false,
            min_periods: self.slow_period,
            ..Default::default()
        });

        Ok(ema_fast - ema_slow)
    }
}
// --- BWMFI (Market Facilitation Index) ---
pub struct BWMFI;

impl BWMFI {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indicator for BWMFI {
    fn alias(&self) -> &'static str { "BWMFI" }
    fn ui_name(&self) -> &'static str { "Market Facilitation Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 } // high, low, volume
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // volume
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        "iBWMFI(_Symbol, _Period)".to_string()
    }
}

impl crate::functions::traits::VectorizedIndicator for BWMFI {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("BWMFI: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("BWMFI: second arg must be low series"),
        };
        let volume = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("BWMFI: third arg must be volume series"),
        };

        Ok((high - low) / volume)
    }
}
