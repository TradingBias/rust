use crate::{
    functions::traits::{Indicator, IndicatorArg, VectorizedIndicator},
    types::ScaleType,
};
use anyhow::{bail, Result};
use polars::{
    lazy::dsl,
    prelude::{lit, when, Duration, EWMOptions, RollingOptionsFixedWindow},
};
use crate::types::DataType;

pub struct RSI {
    pub period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    fn smoothed_ma(&self, series: &dsl::Expr, period: usize) -> Result<dsl::Expr> {
        Ok(series.clone().ewm_mean(
            EWMOptions {
                alpha: 1.0 / period as f64,
                adjust: false,
                min_periods: period,
                ..Default::default()
            }
        ))
    }
}

impl Indicator for RSI {
    fn alias(&self) -> &'static str {
        "RSI"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Relative Strength Index"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Oscillator0_100
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        Some((0.0, 100.0))
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
        format!("iRSI({}, {}, {}, {})", args[0], args[1], args[2], args[3])
    }
}

impl VectorizedIndicator for RSI {
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
        // In Polars 0.51, diff() was removed from Expr. Use shift() instead.
        let delta = series.clone() - series.clone().shift(lit(1));

        // Step 2: Separate gains and losses
        let gains = delta
            .clone()
            .clip(dsl::lit(0.0), dsl::lit(f64::INFINITY));
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
        Self {
            k_period,
            d_period,
            slowing,
        }
    }
}

impl Indicator for Stochastic {
    fn alias(&self) -> &'static str {
        "Stochastic"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Stochastic Oscillator"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Oscillator0_100
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        Some((0.0, 100.0))
    }
    fn arity(&self) -> usize {
        6
    } // high, low, close, k_period, d_period, slowing
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
        format!(
            "iStochastic(_Symbol, _Period, {}, {}, {}, MODE_SMA, STO_LOWHIGH)",
            self.k_period, self.d_period, self.slowing
        )
    }
}

impl VectorizedIndicator for Stochastic {
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

        let options = RollingOptionsFixedWindow {
            window_size: self.k_period as usize,
            min_periods: self.k_period as usize,
            ..Default::default()
        };

        let highest_high = high.rolling_max(options.clone());
        let lowest_low = low.rolling_min(options);

        let percent_k = (close - lowest_low.clone()) / (highest_high - lowest_low) * dsl::lit(100.0);

        let d_options = RollingOptionsFixedWindow {
            window_size: self.d_period as usize,
            min_periods: self.d_period,
            ..Default::default()
        };
        let percent_d = percent_k.rolling_mean(d_options);

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
    fn alias(&self) -> &'static str {
        "CCI"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Commodity Channel Index"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        4
    } // high, low, close, period
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

impl VectorizedIndicator for CCI {
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

        let options = RollingOptionsFixedWindow {
            window_size: self.period as usize,
            min_periods: self.period,
            ..Default::default()
        };

        let typical_price = (high + low + close) / dsl::lit(3.0);
        let sma_tp = typical_price.clone().rolling_mean(options.clone());

        let mean_deviation = (typical_price.clone() - sma_tp.clone())
            .abs()
            .rolling_mean(options);

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
    fn alias(&self) -> &'static str {
        "WilliamsR"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Williams' %R"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Oscillator0_100
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        Some((-100.0, 0.0))
    }
    fn arity(&self) -> usize {
        4
    } // high, low, close, period
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

impl VectorizedIndicator for WilliamsR {
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
        let options = RollingOptionsFixedWindow {
            window_size: self.period as usize,
            min_periods: self.period,
            ..Default::default()
        };

        let highest_high = high.rolling_max(options.clone());
        let lowest_low = low.rolling_min(options);

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
    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Rate of Change"
    }
    fn alias(&self) -> &'static str {
        "ROC"
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
        format!(
            "iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)",
            self.period
        )
    }
}

impl VectorizedIndicator for ROC {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("ROC: first arg must be close series"),
        };

        let prev_close = close.clone().shift(lit(self.period as i64));

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
    fn alias(&self) -> &'static str {
        "AC"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Accelerator Oscillator"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    } // high, low
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

impl VectorizedIndicator for AC {
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

        let options5 = RollingOptionsFixedWindow {
            window_size: 5,
            min_periods: 5,
            ..Default::default()
        };

        let options34 = RollingOptionsFixedWindow {
            window_size: 34,
            min_periods: 34,
            ..Default::default()
        };

        let ao = median_price.clone().rolling_mean(options5.clone())
            - median_price.rolling_mean(options34.clone());

        let ac = ao.clone() - ao.rolling_mean(options5);
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
    fn alias(&self) -> &'static str {
        "AO"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Awesome Oscillator"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        2
    } // high, low
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

impl VectorizedIndicator for AO {
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

        let options5 = RollingOptionsFixedWindow {
            window_size: 5,
            min_periods: 5,
            ..Default::default()
        };

        let options34 = RollingOptionsFixedWindow {
            window_size: 34,
            min_periods: 34,
            ..Default::default()
        };
        let ao = median_price.clone().rolling_mean(options5) - median_price.rolling_mean(options34);

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
    fn alias(&self) -> &'static str {
        "RVI"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Relative Vigor Index"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::OscillatorCentered
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        None
    }
    fn arity(&self) -> usize {
        5
    } // open, high, low, close, period
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

impl VectorizedIndicator for RVI {
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

        let numerator = (close.clone() - open.clone())
            + lit(2.0) * (close.clone().shift(lit(1)) - open.clone().shift(lit(1)))
            + lit(2.0) * (close.clone().shift(lit(2)) - open.clone().shift(lit(2)))
            + (close.shift(lit(3)) - open.shift(lit(3)));
        let denominator = (high.clone() - low.clone())
            + lit(2.0) * (high.clone().shift(lit(1)) - low.clone().shift(lit(1)))
            + lit(2.0) * (high.clone().shift(lit(2)) - low.clone().shift(lit(2)))
            + (high.shift(lit(3)) - low.shift(lit(3)));
        let options = RollingOptionsFixedWindow {
            window_size: self.period as usize,
            min_periods: self.period,
            ..Default::default()
        };

        let rvi =
            numerator.rolling_sum(options.clone()) / denominator.rolling_sum(options.clone());

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
    fn alias(&self) -> &'static str {
        "DeMarker"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "DeMarker Indicator"
    }
    fn scale_type(&self) -> ScaleType {
        ScaleType::Oscillator0_100
    }
    fn value_range(&self) -> Option<(f64, f64)> {
        Some((0.0, 1.0))
    }
    fn arity(&self) -> usize {
        3
    } // high, low, period
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

impl VectorizedIndicator for DeMarker {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: second arg must be low series"),
        };

        let de_max = when(high.clone().gt(high.clone().shift(lit(1))))
            .then(high.clone() - high.shift(lit(1)))
            .otherwise(dsl::lit(0.0));

        let de_min = when(low.clone().lt(low.clone().shift(lit(1))))
            .then(low.clone().shift(lit(1)) - low.clone())
            .otherwise(dsl::lit(0.0));

        let options = RollingOptionsFixedWindow {
            window_size: self.period as usize,
            min_periods: self.period,
            ..Default::default()
        };

        let sma_de_max = de_max.rolling_mean(options.clone());

        let sma_de_min = de_min.rolling_mean(options);

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
    fn alias(&self) -> &'static str {
        "Momentum"
    }

    fn output_type(&self) -> DataType {
        DataType::Float
    }
    fn ui_name(&self) -> &'static str {
        "Momentum Indicator"
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
        format!(
            "iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)",
            self.period
        )
    }
}

impl VectorizedIndicator for Momentum {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Momentum: first arg must be close series"),
        };

        Ok(close.clone() - close.shift(lit(self.period as i64)))
    }
}
