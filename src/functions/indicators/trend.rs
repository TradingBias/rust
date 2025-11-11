use std::collections::VecDeque;
use std::any::Any;
use anyhow::{Result, bail};
use polars::prelude::{DataType as PolarsDataType, Duration};
use polars::lazy::dsl;
use crate::functions::primitives::{MovingAverage, MAMethod, StdDev};
use crate::functions::traits::{Indicator, IndicatorArg, Primitive};
use crate::types::{DataType, ScaleType};

// --- SMA ---
pub struct SMA {
    pub period: usize,
}

pub struct SMAState {
    buffer: VecDeque<f64>,
    period: usize,
}

impl Indicator for SMA {
    fn alias(&self) -> &'static str { "SMA" }
    fn ui_name(&self) -> &'static str { "Simple Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> { vec![DataType::NumericSeries, DataType::Integer] }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("SMA: first arg must be series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("SMA: second arg must be scalar period"),
        };
        let ma = MovingAverage { method: MAMethod::Simple };
        ma.execute(&[series, dsl::lit(period)])
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<SMAState>().unwrap();
        state.buffer.push_back(args[0]);
        if state.buffer.len() > state.period {
            state.buffer.pop_front();
        }
        if state.buffer.len() < state.period {
            return Ok(0.0);
        }
        let sum: f64 = state.buffer.iter().sum();
        Ok(sum / state.period as f64)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(SMAState {
            buffer: VecDeque::with_capacity(self.period),
            period: self.period,
        })
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMA({}, {}, {}, 0, MODE_SMA, {}, {})", args[0], args[1], self.period, args[2], args[3])
    }
}

// --- EMA ---
pub struct EMA {
    pub period: usize,
}

pub struct EMAState {
    period: usize,
    prev_ema: Option<f64>,
    init_buffer: VecDeque<f64>,
}

impl Indicator for EMA {
    fn alias(&self) -> &'static str { "EMA" }
    fn ui_name(&self) -> &'static str { "Exponential Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> { vec![DataType::NumericSeries, DataType::Integer] }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("EMA: first arg must be series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("EMA: second arg must be scalar period"),
        };
        let ma = MovingAverage { method: MAMethod::Exponential };
        ma.execute(&[series, dsl::lit(period)])
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<EMAState>().unwrap();
        let price = args[0];

        if state.prev_ema.is_none() {
            state.init_buffer.push_back(price);
            if state.init_buffer.len() < state.period {
                return Ok(price);
            }
            let sum: f64 = state.init_buffer.iter().sum();
            state.prev_ema = Some(sum / state.period as f64);
        }

        let alpha = 2.0 / (state.period as f64 + 1.0);
        let ema = alpha * price + (1.0 - alpha) * state.prev_ema.unwrap();
        state.prev_ema = Some(ema);
        Ok(ema)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(EMAState {
            period: self.period,
            prev_ema: None,
            init_buffer: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMA({}, {}, {}, 0, MODE_EMA, {}, {})", args[0], args[1], self.period, args[2], args[3])
    }
}

// --- MACD ---
pub struct MACD {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

pub struct MACDState {
    fast_ema_state: EMAState,
    slow_ema_state: EMAState,
    signal_ema_state: EMAState,
    fast_ema_indicator: EMA,
    slow_ema_indicator: EMA,
    signal_ema_indicator: EMA,
}

impl Indicator for MACD {
    fn alias(&self) -> &'static str { "MACD" }
    fn ui_name(&self) -> &'static str { "Moving Average Convergence/Divergence" }
    fn scale_type(&self) -> ScaleType { ScaleType::OscillatorCentered }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 4 }
    fn input_types(&self) -> Vec<DataType> { vec![DataType::NumericSeries, DataType::Integer, DataType::Integer, DataType::Integer] }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("MACD: first arg must be series"),
        };
        let ema_fast_primitive = MovingAverage { method: MAMethod::Exponential };
        let ema_slow_primitive = MovingAverage { method: MAMethod::Exponential };

        let ema_fast = ema_fast_primitive.execute(&[series.clone(), dsl::lit(self.fast_period as i64)])?;
        let ema_slow = ema_slow_primitive.execute(&[series, dsl::lit(self.slow_period as i64)])?;
        
        let macd_line = ema_fast - ema_slow;
        
        Ok(macd_line)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<MACDState>().unwrap();
        let price = args[0];

        let fast_ema = state.fast_ema_indicator.calculate_stateful(&[price], &mut state.fast_ema_state)?;
        let slow_ema = state.slow_ema_indicator.calculate_stateful(&[price], &mut state.slow_ema_state)?;

        let macd_line = fast_ema - slow_ema;
        
        let _signal_line = state.signal_ema_indicator.calculate_stateful(&[macd_line], &mut state.signal_ema_state)?;

        Ok(macd_line)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(MACDState {
            fast_ema_state: EMAState { period: self.fast_period, prev_ema: None, init_buffer: VecDeque::with_capacity(self.fast_period) },
            slow_ema_state: EMAState { period: self.slow_period, prev_ema: None, init_buffer: VecDeque::with_capacity(self.slow_period) },
            signal_ema_state: EMAState { period: self.signal_period, prev_ema: None, init_buffer: VecDeque::with_capacity(self.signal_period) },
            fast_ema_indicator: EMA { period: self.fast_period },
            slow_ema_indicator: EMA { period: self.slow_period },
            signal_ema_indicator: EMA { period: self.signal_period },
        })
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iMACD({}, {}, {}, {}, {}, {}, {}, {})", args[0], args[1], self.fast_period, self.slow_period, self.signal_period, args[2], args[3], args[4])
    }
}


// --- Bollinger Bands ---
pub struct BollingerBands {
    pub period: usize,
    pub deviation: f64,
}

pub struct BBState {
    buffer: VecDeque<f64>,
    period: usize,
}

impl Indicator for BollingerBands {
    fn alias(&self) -> &'static str { "BB" }
    fn ui_name(&self) -> &'static str { "Bollinger Bands" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 }
    fn input_types(&self) -> Vec<DataType> { vec![DataType::NumericSeries, DataType::Integer, DataType::Float] }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("BB: first arg must be series"),
        };
        let period = self.period as i64;
        let deviation = self.deviation;

        let sma = MovingAverage { method: MAMethod::Simple };
        let std_dev = StdDev;

        let middle_band = sma.execute(&[series.clone(), dsl::lit(period)])?;
        let std_dev_val = std_dev.execute(&[series, dsl::lit(period)])?;

        let upper_band = middle_band.clone() + (dsl::lit(deviation) * std_dev_val.clone());
        let _lower_band = middle_band.clone() - (dsl::lit(deviation) * std_dev_val);

        Ok(upper_band)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<BBState>().unwrap();
        state.buffer.push_back(args[0]);
        if state.buffer.len() > state.period {
            state.buffer.pop_front();
        }

        if state.buffer.len() < state.period {
            return Ok(args[0]);
        }

        let sum: f64 = state.buffer.iter().sum();
        let mean = sum / state.period as f64;

        let std_dev_sum: f64 = state.buffer.iter().map(|x| (x - mean).powi(2)).sum();
        let std_dev = (std_dev_sum / state.period as f64).sqrt();

        let upper_band = mean + self.deviation * std_dev;

        Ok(upper_band)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(BBState {
            buffer: VecDeque::with_capacity(self.period),
            period: self.period,
        })
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iBands({}, {}, {}, {}, 0, {}, {}, {})", args[0], args[1], self.period, self.deviation, args[2], args[3], args[4])
    }
}