use std::collections::VecDeque;
use std::any::Any;
use anyhow::{Result, bail};
use polars::prelude::{EWMOptions, when}; // Added 'when'
use polars::lazy::dsl;
use polars::series::ops::NullBehavior;
use crate::functions::traits::{Indicator, IndicatorArg, VectorizedIndicator}; // Added VectorizedIndicator
use crate::types::{DataType, ScaleType};
// Removed import for SMA and SMAState from trend

pub struct RSI {
    pub period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    fn smoothed_ma(&self, series: &dsl::Expr, period: usize) -> Result<dsl::Expr> {
        Ok(series.clone().ewm_mean(EWMOptions {
            alpha: 1.0 / period as f64,
            adjust: false,
            min_periods: period,
            ..Default::default()
        }))
    }
}

impl Indicator for RSI {
    fn alias(&self) -> &'static str { "RSI" }
    fn ui_name(&self) -> &'static str { "Relative Strength Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 2 }
    
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iRSI({}, {}, {}, {})", args[0], args[1], args[2], args[3])
    }
}

impl crate::functions::traits::VectorizedIndicator for RSI {
    /// VECTORIZED: Calculate RSI over entire series
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RSI: first arg must be series"),
        };
        
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("RSI: second arg must be scalar"),
        };
        
        // Step 1: Calculate price changes
        let delta = series.diff(1, NullBehavior::Ignore);
        
        // Step 2: Separate gains and losses
        let gains = delta.clone().clip(dsl::lit(0.0), dsl::lit(f64::INFINITY));
        let losses = (delta.clip(dsl::lit(f64::NEG_INFINITY), dsl::lit(0.0))).abs();
        
        // Step 3: Calculate average gains and losses using SMMA
        let avg_gains = self.smoothed_ma(&gains, period)?;
        let avg_losses = self.smoothed_ma(&losses, period)?;
        
        // Step 4: Calculate RS and RSI
        let rs = avg_gains.clone() / avg_losses.clone();
        let rsi = dsl::lit(100.0) - (dsl::lit(100.0) / (dsl::lit(1.0) + rs));
        
        Ok(rsi)
    }
}
// --- Stochastic ---
pub struct Stochastic {
    pub k_period: usize,
    pub d_period: usize,
    pub slowing: usize,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize, slowing: usize) -> Self {
        Self { k_period, d_period, slowing }
    }
}

impl Indicator for Stochastic {
    fn alias(&self) -> &'static str { "Stochastic" }
    fn ui_name(&self) -> &'static str { "Stochastic Oscillator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 6 } // high, low, close, k_period, d_period, slowing
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // k_period
            DataType::Integer,       // d_period
            DataType::Integer,       // slowing
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iStochastic(_Symbol, _Period, {}, {}, {}, MODE_SMA, STO_LOWHIGH)", self.k_period, self.d_period, self.slowing)
    }
}

impl crate::functions::traits::VectorizedIndicator for Stochastic {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Stochastic: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Stochastic: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Stochastic: third arg must be close series"),
        };

        let highest_high = high.rolling_max(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.k_period,
            ..Default::default()
        });
        let lowest_low = low.rolling_min(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.k_period,
            ..Default::default()
        });

        let percent_k = (close - lowest_low.clone()) / (highest_high - lowest_low) * dsl::lit(100.0);
        let percent_d = percent_k.rolling_mean(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.d_period,
            ..Default::default()
        });

        Ok(percent_d)
    }
}
// --- CCI (Commodity Channel Index) ---
pub struct CCI {
    pub period: usize,
}

impl CCI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for CCI {
    fn alias(&self) -> &'static str { "CCI" }
    fn ui_name(&self) -> &'static str { "Commodity Channel Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 4 } // high, low, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iCCI(_Symbol, _Period, {}, PRICE_TYPICAL)", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for CCI {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("CCI: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("CCI: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("CCI: third arg must be close series"),
        };

        let typical_price = (high + low + close) / dsl::lit(3.0);
        let sma_tp = typical_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        let mean_deviation = (typical_price.clone() - sma_tp.clone()).abs().rolling_mean(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        let cci = (typical_price - sma_tp) / (dsl::lit(0.015) * mean_deviation);
        Ok(cci)
    }
}
// --- Williams' %R ---
pub struct WilliamsR {
    pub period: usize,
}

impl WilliamsR {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for WilliamsR {
    fn alias(&self) -> &'static str { "WilliamsR" }
    fn ui_name(&self) -> &'static str { "Williams' %R" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((-100.0, 0.0)) }
    fn arity(&self) -> usize { 4 } // high, low, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iWPR(_Symbol, _Period, {})", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for WilliamsR {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("WilliamsR: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("WilliamsR: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("WilliamsR: third arg must be close series"),
        };

        let highest_high = high.rolling_max(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });
        let lowest_low = low.rolling_min(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        Ok(((highest_high.clone() - close) / (highest_high - lowest_low)) * dsl::lit(-100.0))
    }
}
// --- ROC (Rate of Change) ---
pub struct ROC {
    pub period: usize,
}

impl ROC {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for ROC {
    fn alias(&self) -> &'static str { "ROC" }
    fn ui_name(&self) -> &'static str { "Rate of Change" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for ROC {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("ROC: first arg must be close series"),
        };

        let prev_close = close.shift(self.period);

        Ok(((close - prev_close.clone()) / prev_close) * dsl::lit(100.0))
    }
}

// --- AC (Accelerator Oscillator) ---
pub struct AC;

impl AC {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indicator for AC {
    fn alias(&self) -> &'static str { "AC" }
    fn ui_name(&self) -> &'static str { "Accelerator Oscillator" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // high, low
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        "iAC(_Symbol, _Period)".to_string()
    }
}

impl crate::functions::traits::VectorizedIndicator for AC {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("AC: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("AC: second arg must be low series"),
        };

        let median_price = (high + low) / dsl::lit(2.0);
        let ao = median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 34, ..Default::default() });

        let ac = ao.clone() - ao.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() });
        Ok(ac)
    }
}
// --- AO (Awesome Oscillator) ---
pub struct AO;

impl AO {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indicator for AO {
    fn alias(&self) -> &'static str { "AO" }
    fn ui_name(&self) -> &'static str { "Awesome Oscillator" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // high, low
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        "iAO(_Symbol, _Period)".to_string()
    }
}

impl crate::functions::traits::VectorizedIndicator for AO {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("AO: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("AO: second arg must be low series"),
        };

        let median_price = (high + low) / dsl::lit(2.0);
        let ao = median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 34, ..Default::default() });

        Ok(ao)
    }
}
// --- RVI (Relative Vigor Index) ---
pub struct RVI {
    pub period: usize,
}

impl RVI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for RVI {
    fn alias(&self) -> &'static str { "RVI" }
    fn ui_name(&self) -> &'static str { "Relative Vigor Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 5 } // open, high, low, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // open
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iRVI(_Symbol, _Period, {})", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for RVI {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let open = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RVI: first arg must be open series"),
        };
        let high = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RVI: second arg must be high series"),
        };
        let low = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RVI: third arg must be low series"),
        };
        let close = match &args[3] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RVI: fourth arg must be close series"),
        };

        let numerator = (close.clone() - open.clone()) + 2.0 * (close.shift(1) - open.shift(1)) + 2.0 * (close.shift(2) - open.shift(2)) + (close.shift(3) - open.shift(3));
        let denominator = (high.clone() - low.clone()) + 2.0 * (high.shift(1) - low.shift(1)) + 2.0 * (high.shift(2) - low.shift(2)) + (high.shift(3) - low.shift(3));

        let rvi = numerator.rolling_sum(polars::prelude::RollingOptionsFixedWindow { window_size: self.period, ..Default::default() }) /
                  denominator.rolling_sum(polars::prelude::RollingOptionsFixedWindow { window_size: self.period, ..Default::default() });

        Ok(rvi)
    }
}
// --- DeMarker ---
pub struct DeMarker {
    pub period: usize,
}

impl DeMarker {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for DeMarker {
    fn alias(&self) -> &'static str { "DeMarker" }
    fn ui_name(&self) -> &'static str { "DeMarker Indicator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 1.0)) }
    fn arity(&self) -> usize { 3 } // high, low, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iDeMarker(_Symbol, _Period, {})", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for DeMarker {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: second arg must be low series"),
        };

        let de_max = when(high.clone().gt(high.shift(1)))
            .then(high.clone() - high.shift(1))
            .otherwise(dsl::lit(0.0));

        let de_min = when(low.clone().lt(low.shift(1)))
            .then(low.shift(1) - low.clone())
            .otherwise(dsl::lit(0.0));

        let sma_de_max = de_max.rolling_mean(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        let sma_de_min = de_min.rolling_mean(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        });

        Ok(sma_de_max.clone() / (sma_de_max + sma_de_min))
    }
}
// --- Momentum ---
pub struct Momentum {
    pub period: usize,
}

impl Momentum {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for Momentum {
    fn alias(&self) -> &'static str { "Momentum" }
    fn ui_name(&self) -> &'static str { "Momentum Indicator" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}

impl crate::functions::traits::VectorizedIndicator for Momentum {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Momentum: first arg must be close series"),
        };

        Ok(close.clone() - close.shift(self.period))
    }
}