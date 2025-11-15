use crate::{
    functions::{
        primitives::{MAMethod, MovingAverage},
        traits::{Indicator, IndicatorArg, Primitive},
    },
    types::{DataType, ScaleType},
    types, // Add this line
};
use anyhow::{bail, Result};
use polars::{lazy::dsl, prelude::{lit}};
use std::any::Any;

// --- SMA ---
pub struct SMA {
    pub period: usize,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for SMA {
    fn alias(&self) -> &'static str {
        "SMA"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Simple Moving Average"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!(
            "iMA({}, {}, {}, 0, MODE_SMA, {}, {})",
            args[0], args[1], self.period, args[2], args[3]
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for SMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("SMA: first arg must be series"),
        };

        // Use period from struct
        use polars::prelude::RollingOptionsFixedWindow;
        let options = RollingOptionsFixedWindow {
            window_size: self.period,
            min_periods: self.period,
            ..Default::default()
        };

        Ok(series.rolling_mean(options))
    }
}

// --- EMA ---
pub struct EMA {
    pub period: usize,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for EMA {
    fn alias(&self) -> &'static str {
        "EMA"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Exponential Moving Average"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!(
            "iMA({}, {}, {}, 0, MODE_EMA, {}, {})",
            args[0], args[1], self.period, args[2], args[3]
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for EMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("EMA: first arg must be series"),
        };

        // Use period from struct
        use polars::prelude::EWMOptions;
        let alpha = 2.0 / (self.period as f64 + 1.0);
        let options = EWMOptions {
            alpha,
            adjust: false,
            min_periods: self.period,
            ..Default::default()
        };

        Ok(series.ewm_mean(options))
    }
}

// --- MACD ---
pub struct MACD {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl MACD {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
        }
    }
}

impl Indicator for MACD {
    fn alias(&self) -> &'static str {
        "MACD"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Moving Average Convergence/Divergence"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        4
    }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,
            DataType::Integer,
            DataType::Integer,
            DataType::Integer,
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!(
            "iMACD({}, {}, {}, {}, {}, {}, {}, {})",
            args[0],
            args[1],
            self.fast_period,
            self.slow_period,
            self.signal_period,
            args[2],
            args[3],
            args[4]
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for MACD {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MACD: first arg must be series"),
        };

        // Calculate EMAs directly using Polars
        use polars::prelude::EWMOptions;

        let fast_alpha = 2.0 / (self.fast_period as f64 + 1.0);
        let fast_options = EWMOptions {
            alpha: fast_alpha,
            adjust: false,
            min_periods: self.fast_period,
            ..Default::default()
        };

        let slow_alpha = 2.0 / (self.slow_period as f64 + 1.0);
        let slow_options = EWMOptions {
            alpha: slow_alpha,
            adjust: false,
            min_periods: self.slow_period,
            ..Default::default()
        };

        let ema_fast = series.clone().ewm_mean(fast_options);
        let ema_slow = series.ewm_mean(slow_options);

        let macd_line = ema_fast - ema_slow;

        Ok(macd_line)
    }
}

// --- Bollinger Bands ---
pub struct BollingerBands {
    pub period: usize,
    pub deviation: f64,
}

impl BollingerBands {
    pub fn new(period: usize, deviation: f64) -> Self {
        Self { period, deviation }
    }
}

impl Indicator for BollingerBands {
    fn alias(&self) -> &'static str {
        "BB"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Bollinger Bands"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        3
    }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,
            DataType::Integer,
            DataType::Float,
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!(
            "iBands({}, {}, {}, {}, 0, {}, {}, {})",
            args[0], args[1], self.period, self.deviation, args[2], args[3], args[4]
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for BollingerBands {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("BB: first arg must be series"),
        };

        // Calculate SMA and StdDev directly using Polars
        use polars::prelude::RollingOptionsFixedWindow;

        let options = RollingOptionsFixedWindow {
            window_size: self.period,
            min_periods: self.period,
            ..Default::default()
        };

        let middle_band = series.clone().rolling_mean(options.clone());
        let std_dev_val = series.rolling_std(options);

        let upper_band = middle_band.clone() + (dsl::lit(self.deviation) * std_dev_val.clone());
        let _lower_band = middle_band - (dsl::lit(self.deviation) * std_dev_val);

        Ok(upper_band)
    }
}

// --- Envelopes ---
pub struct Envelopes {
    pub period: usize,
    pub deviation: f64,
}

impl Envelopes {
    pub fn new(period: usize, deviation: f64) -> Self {
        Self { period, deviation }
    }
}

impl Indicator for Envelopes {
    fn alias(&self) -> &'static str {
        "Envelopes"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Envelopes"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        3
    } // close, period, deviation
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::Integer,       // period
            DataType::Float,         // deviation
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!(
            "iEnvelopes(_Symbol, _Period, {}, MODE_SMA, 0, PRICE_CLOSE, {})",
            self.period, self.deviation
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for Envelopes {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Envelopes: first arg must be close series"),
        };

        let ma = MovingAverage {
            method: MAMethod::Simple,
        };
        let middle_line = ma.execute(&[close, dsl::lit(self.period as i64)])?;

        let upper_band = middle_line.clone() * (dsl::lit(1.0) + dsl::lit(self.deviation));
        let _lower_band = middle_line * (dsl::lit(1.0) - dsl::lit(self.deviation));

        Ok(upper_band)
    }
}

// --- SAR (Parabolic SAR) ---
pub struct SAR {
    pub step: f64,
    pub max: f64,
}

impl SAR {
    pub fn new(step: f64, max: f64) -> Self {
        Self { step, max }
    }
}

pub struct SARState {
    step: f64,
    max: f64,
    sar: f64,
    ep: f64,
    af: f64,
    is_rising: bool,
}

impl Indicator for SAR {
    fn alias(&self) -> &'static str {
        "SAR"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Parabolic SAR"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        4
    } // high, low, step, max
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::Float,         // step
            DataType::Float,         // max
        ]
    }

    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Stateful
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iSAR(_Symbol, _Period, {}, {})", self.step, self.max)
    }
}

impl crate::functions::traits::StatefulIndicator for SAR {
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<SARState>().unwrap();
        let high = args[0];
        let low = args[1];

        if state.is_rising {
            state.sar = state.sar + state.af * (state.ep - state.sar);
            if high > state.ep {
                state.ep = high;
                state.af = (state.af + state.step).min(state.max);
            }
            if low < state.sar {
                state.is_rising = false;
                state.sar = state.ep;
                state.ep = low;
                state.af = state.step;
            }
        } else {
            state.sar = state.sar - state.af * (state.sar - state.ep);
            if low < state.ep {
                state.ep = low;
                state.af = (state.af + state.step).min(state.max);
            }
            if high > state.sar {
                state.is_rising = true;
                state.sar = state.ep;
                state.ep = high;
                state.af = state.step;
            }
        }

        Ok(state.sar)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(SARState {
            step: self.step,
            max: self.max,
            sar: 0.0,
            ep: 0.0,
            af: self.step,
            is_rising: true,
        })
    }
}
// --- Bears Power ---
pub struct Bears {
    pub period: usize, // Add period to struct for new()
}

impl Bears {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for Bears {
    fn alias(&self) -> &'static str {
        "Bears"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Bears Power"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    } // Changed from Price to OscillatorCentered
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        3
    } // low, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // period (ignored, uses self.period)
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!(
            "iBearsPower(_Symbol, _Period, {}, PRICE_CLOSE)",
            self.period
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for Bears {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let low = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bears: first arg must be low series"),
        };
        let close = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bears: second arg must be close series"),
        };

        let ema = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema_val = ema.execute(&[close, dsl::lit(self.period as i64)])?;

        Ok(low - ema_val)
    }
}

// --- Bulls Power ---
pub struct Bulls {
    pub period: usize, // Add period to struct for new()
}

impl Bulls {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for Bulls {
    fn alias(&self) -> &'static str {
        "Bulls"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Bulls Power"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    } // Changed from Price to OscillatorCentered
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        3
    } // high, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // close
            DataType::Integer,       // period (ignored, uses self.period)
        ]
    }
    fn calculation_mode(&self) -> crate::functions::traits::CalculationMode {
        crate::functions::traits::CalculationMode::Vectorized
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        format!(
            "iBullsPower(_Symbol, _Period, {}, PRICE_CLOSE)",
            self.period
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for Bulls {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bulls: first arg must be high series"),
        };
        let close = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bulls: second arg must be close series"),
        };

        let ema = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema_val = ema.execute(&[close, dsl::lit(self.period as i64)])?;

        Ok(high - ema_val)
    }
}
// --- DEMA (Double Exponential Moving Average) ---
pub struct DEMA {
    pub period: usize,
}

impl DEMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for DEMA {
    fn alias(&self) -> &'static str {
        "DEMA"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Double Exponential Moving Average"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    } // close, period
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
        format!(
            "iMAOnArray(DEMA_buffer, 0, {}, 0, MODE_EMA, 0)",
            self.period
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for DEMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DEMA: first arg must be close series"),
        };

        let ema1 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema1_val = ema1.execute(&[close, dsl::lit(self.period as i64)])?;

        let ema2 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema2_val = ema2.execute(&[ema1_val.clone(), dsl::lit(self.period as i64)])?;

        Ok(dsl::lit(2.0) * ema1_val - ema2_val)
    }
}

// --- TEMA (Triple Exponential Moving Average) ---
pub struct TEMA {
    pub period: usize,
}

impl TEMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for TEMA {
    fn alias(&self) -> &'static str {
        "TEMA"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Triple Exponential Moving Average"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Price
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    } // close, period
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
        format!(
            "iMAOnArray(TEMA_buffer, 0, {}, 0, MODE_EMA, 0)",
            self.period
        )
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for TEMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("TEMA: first arg must be close series"),
        };

        let ema1 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema1_val = ema1.execute(&[close, dsl::lit(self.period as i64)])?;

        let ema2 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema2_val = ema2.execute(&[ema1_val.clone(), dsl::lit(self.period as i64)])?;

        let ema3 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema3_val = ema3.execute(&[ema2_val.clone(), dsl::lit(self.period as i64)])?;

        Ok(dsl::lit(3.0) * (ema1_val - ema2_val) + ema3_val)
    }
}
// --- TriX (Triple Exponential Average) ---
pub struct TriX {
    pub period: usize,
}

impl TriX {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Indicator for TriX {
    fn alias(&self) -> &'static str {
        "TriX"
    }

    fn output_type(&self) -> types::DataType {
        types::DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Triple Exponential Average"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    } // close, period
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
        format!("iTriX(_Symbol, _Period, {})", self.period)
    }

    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<dsl::Expr>> {
        Some(crate::functions::traits::VectorizedIndicator::calculate_vectorized(self, args))
    }
}

impl crate::functions::traits::VectorizedIndicator for TriX {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("TriX: first arg must be close series"),
        };

        let ema1 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema1_val = ema1.execute(&[close, dsl::lit(self.period as i64)])?;

        let ema2 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema2_val = ema2.execute(&[ema1_val, dsl::lit(self.period as i64)])?;

        let ema3 = MovingAverage {
            method: MAMethod::Exponential,
        };
        let ema3_val = ema3.execute(&[ema2_val, dsl::lit(self.period as i64)])?;
        let prev_ema3 = ema3_val.clone().shift(lit(1));

        Ok((ema3_val - prev_ema3.clone()) / prev_ema3)
    }
}
