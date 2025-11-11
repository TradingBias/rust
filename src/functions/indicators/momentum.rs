use std::collections::VecDeque;
use std::any::Any;
use anyhow::{Result, bail};
use polars::prelude::{EWMOptions};
use polars::lazy::dsl;
use polars::series::ops::NullBehavior;
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};
use crate::functions::indicators::trend::{SMA, SMAState};

/// State for stateful RSI calculation
pub struct RSIState {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    prev_close: Option<f64>,
}
pub struct RSI {
    pub period: usize,
}

impl RSI {
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
        let delta = series.clone() - series.shift(dsl::lit(1));
        
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
    
    /// STATEFUL: Calculate RSI for single bar
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<RSIState>()
            .ok_or_else(|| anyhow::anyhow!("Invalid state type for RSI"))?;
        
        let close = args[0];
        
        if state.prev_close.is_none() {
            state.prev_close = Some(close);
            return Ok(50.0); 
        }
        
        let prev_close = state.prev_close.unwrap();
        let change = close - prev_close;
        
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };
        
        state.gains.push_back(gain);
        state.losses.push_back(loss);
        
        if state.gains.len() > self.period {
            state.gains.pop_front();
            state.losses.pop_front();
        }
        
        if state.gains.len() < self.period {
            state.prev_close = Some(close);
            return Ok(50.0);
        }
        
        if state.avg_gain.is_none() {
            let sum_gain: f64 = state.gains.iter().sum();
            let sum_loss: f64 = state.losses.iter().sum();
            state.avg_gain = Some(sum_gain / state.period as f64);
            state.avg_loss = Some(sum_loss / state.period as f64);
        } else {
            let period = state.period as f64;
            state.avg_gain = Some(
                (state.avg_gain.unwrap() * (period - 1.0) + gain) / period
            );
            state.avg_loss = Some(
                (state.avg_loss.unwrap() * (period - 1.0) + loss) / period
            );
        }
        
        let avg_gain = state.avg_gain.unwrap();
        let avg_loss = state.avg_loss.unwrap();
        
        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        
        state.prev_close = Some(close);
        Ok(rsi)
    }
    
    fn init_state(&self) -> Box<dyn Any> {
        Box::new(RSIState {
            period: self.period,
            gains: VecDeque::with_capacity(self.period),
            losses: VecDeque::with_capacity(self.period),
            avg_gain: None,
            avg_loss: None,
            prev_close: None,
        })
    }
    
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iRSI({}, {}, {}, {})", args[0], args[1], args[2], args[3])
    }
}

// --- Stochastic ---
pub struct Stochastic {
    pub k_period: usize,
    pub d_period: usize,
    pub slowing: usize,
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

        let k_period = match &args[3] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Stochastic: fourth arg must be scalar k_period"),
        };
        let d_period = match &args[4] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Stochastic: fifth arg must be scalar d_period"),
        };
        let slowing = match &args[5] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Stochastic: sixth arg must be scalar slowing"),
        };

        let highest_high = high.rolling_max(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(k_period as i64),
            ..Default::default()
        });
        let lowest_low = low.rolling_min(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(k_period as i64),
            ..Default::default()
        });

        let percent_k = (close - lowest_low.clone()) / (highest_high - lowest_low) * dsl::lit(100.0);
        let slowed_k = percent_k.rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(slowing as i64),
            ..Default::default()
        });
        let percent_d = slowed_k.rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(d_period as i64),
            ..Default::default()
        });

        Ok(percent_d)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Stochastic is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iStochastic(_Symbol, _Period, {}, {}, {}, MODE_SMA, STO_LOWHIGH)", self.k_period, self.d_period, self.slowing)
    }
}

// --- CCI (Commodity Channel Index) ---
pub struct CCI {
    pub period: usize,
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

        let period = match &args[3] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("CCI: fourth arg must be scalar period"),
        };

        let typical_price = (high + low + close) / dsl::lit(3.0);
        let sma_tp = typical_price.clone().rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        let mean_deviation = (typical_price.clone() - sma_tp.clone()).abs().rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        let cci = (typical_price - sma_tp) / (dsl::lit(0.015) * mean_deviation);
        Ok(cci)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("CCI is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iCCI(_Symbol, _Period, {}, PRICE_TYPICAL)", self.period)
    }
}

// --- Williams' %R ---
pub struct WilliamsR {
    pub period: usize,
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

        let period = match &args[3] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("WilliamsR: fourth arg must be scalar period"),
        };

        let highest_high = high.rolling_max(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });
        let lowest_low = low.rolling_min(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        Ok(((highest_high.clone() - close) / (highest_high - lowest_low)) * dsl::lit(-100.0))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("WilliamsR is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iWPR(_Symbol, _Period, {})", self.period)
    }
}

// --- ROC (Rate of Change) ---
pub struct ROC {
    pub period: usize,
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

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("ROC: first arg must be close series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("ROC: second arg must be scalar period"),
        };

        let prev_close = close.shift(dsl::lit(period));

        Ok(((close - prev_close.clone()) / prev_close) * dsl::lit(100.0))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("ROC is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}

// --- AC (Accelerator Oscillator) ---

// --- AC (Accelerator Oscillator) ---
pub struct AC;

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
        let ao = median_price.clone().rolling_mean(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(5), ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(34), ..Default::default() });

        let ac = ao.clone() - ao.rolling_mean(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(5), ..Default::default() });
        Ok(ac)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("AC is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iAC(_Symbol, _Period)".to_string()
    }
}

// --- AO (Awesome Oscillator) ---
pub struct AO;
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
        let ao = median_price.clone().rolling_mean(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(5), ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(34), ..Default::default() });

        Ok(ao)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("AO is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iAO(_Symbol, _Period)".to_string()
    }
}

// --- RVI (Relative Vigor Index) ---
pub struct RVI {
    pub period: usize,
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

        let period = match &args[4] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("RVI: fifth arg must be scalar period"),
        };

        let numerator = (close - open).rolling_sum(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(period as i64), ..Default::default() });
        let denominator = (high - low).rolling_sum(polars::prelude::RollingOptions { window_size: polars::prelude::Duration::new(period as i64), ..Default::default() });

        let rvi_num = numerator / dsl::lit(period as f64);
        let rvi_den = denominator / dsl::lit(period as f64);

        Ok(rvi_num / rvi_den)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("RVI is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iRVI(_Symbol, _Period, {})", self.period)
    }
}

// --- DeMarker ---
pub struct DeMarker {
    pub period: usize,
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

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("DeMarker: second arg must be low series"),
        };

        let period = match &args[2] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("DeMarker: third arg must be scalar period"),
        };

        let de_max = when(high.clone().gt(high.shift(dsl::lit(1))))
            .then(high.clone() - high.shift(dsl::lit(1)))
            .otherwise(dsl::lit(0.0));

        let de_min = when(low.clone().lt(low.shift(dsl::lit(1))))
            .then(low.shift(dsl::lit(1)) - low.clone())
            .otherwise(dsl::lit(0.0));

        let sma_de_max = de_max.rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        let sma_de_min = de_min.rolling_mean(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        Ok(sma_de_max.clone() / (sma_de_max + sma_de_min))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("DeMarker is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iDeMarker(_Symbol, _Period, {})", self.period)
    }
}

// --- Momentum ---
pub struct Momentum {
    pub period: usize,
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

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Momentum: first arg must be close series"),
        };

        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("Momentum: second arg must be scalar period"),
        };

        Ok(close.clone() - close.shift(dsl::lit(period)))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Momentum is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}