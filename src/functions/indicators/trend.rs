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
        let fast_period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("MACD: second arg must be scalar fast_period"),
        };
        let slow_period = match &args[2] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("MACD: third arg must be scalar slow_period"),
        };

        let ema_fast_primitive = MovingAverage { method: MAMethod::Exponential };
        let ema_slow_primitive = MovingAverage { method: MAMethod::Exponential };

        let ema_fast = ema_fast_primitive.execute(&[series.clone(), dsl::lit(fast_period)])?;
        let ema_slow = ema_slow_primitive.execute(&[series, dsl::lit(slow_period)])?;
        
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
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("BB: second arg must be scalar period"),
        };
        let deviation = match &args[2] {
            IndicatorArg::Scalar(d) => *d,
            _ => bail!("BB: third arg must be scalar deviation"),
        };

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

// --- Envelopes ---
pub enum EnvelopeBand {
    Upper,
    Lower,
}

pub struct Envelopes {
    pub period: usize,
    pub deviation: f64,
    pub band: EnvelopeBand,
}

impl Indicator for Envelopes {
    fn alias(&self) -> &'static str { "Envelopes" }
    fn ui_name(&self) -> &'static str { "Envelopes" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 } // close, period, deviation
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // close
            DataType::Integer,       // period
            DataType::Float,         // deviation
        ]
    }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let close = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Envelopes: first arg must be close series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("Envelopes: second arg must be scalar period"),
        };

        let ma = MovingAverage { method: MAMethod::Simple };
        let middle_line = ma.execute(&[close, dsl::lit(period)])?;

        let band = match self.band {
            EnvelopeBand::Upper => middle_line.clone() * (dsl::lit(1.0) + dsl::lit(self.deviation)),
            EnvelopeBand::Lower => middle_line.clone() * (dsl::lit(1.0) - dsl::lit(self.deviation)),
        };

        Ok(band)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Envelopes is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iEnvelopes(_Symbol, _Period, {}, MODE_SMA, 0, PRICE_CLOSE, {})", self.period, self.deviation)
    }
}

// --- SAR (Parabolic SAR) ---
pub struct SAR {
    pub step: f64,
    pub max: f64,
}

pub struct SARState {
    step: f64,
    max: f64,
    sar: Option<f64>,
    ep: Option<f64>,
    af: f64,
    is_rising: bool,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
}

impl Indicator for SAR {
    fn alias(&self) -> &'static str { "SAR" }
    fn ui_name(&self) -> &'static str { "Parabolic SAR" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 4 } // high, low, step, max
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

    fn calculate_vectorized(&self, _args: &[IndicatorArg]) -> Result<dsl::Expr> {
        bail!("SAR is a stateful indicator and does not have a vectorized implementation.")
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<SARState>().unwrap();
        let high = args[0];
        let low = args[1];

        state.highs.push_back(high);
        state.lows.push_back(low);

        if state.sar.is_none() {
            if state.highs.len() < 2 {
                return Ok(low); // Not enough data yet
            }
            let prev_high = state.highs[0];
            let prev_low = state.lows[0];

            state.is_rising = high > prev_high;
            if state.is_rising {
                state.ep = Some(high);
                state.sar = Some(prev_low);
            } else {
                state.ep = Some(low);
                state.sar = Some(prev_high);
            }
        }

        let mut sar = state.sar.unwrap();
        let mut ep = state.ep.unwrap();

        if state.is_rising {
            sar = sar + state.af * (ep - sar);
            if high > ep {
                ep = high;
                state.af = (state.af + state.step).min(state.max);
            }
            if low < sar {
                state.is_rising = false;
                sar = ep;
                ep = low;
                state.af = state.step;
            }
        } else {
            sar = sar - state.af * (sar - ep);
            if low < ep {
                ep = low;
                state.af = (state.af + state.step).min(state.max);
            }
            if high > sar {
                state.is_rising = true;
                sar = ep;
                ep = high;
                state.af = state.step;
            }
        }

        state.sar = Some(sar);
        state.ep = Some(ep);

        Ok(sar)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(SARState {
            step: self.step,
            max: self.max,
            sar: None,
            ep: None,
            af: self.step,
            is_rising: true,
            highs: VecDeque::with_capacity(2),
            lows: VecDeque::with_capacity(2),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iSAR(_Symbol, _Period, {}, {})", self.step, self.max)
    }
}

// --- Bears Power ---
pub struct Bears {
    pub period: usize,
}

impl Indicator for Bears {
    fn alias(&self) -> &'static str { "Bears" }
    fn ui_name(&self) -> &'static str { "Bears Power" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 } // low, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let low = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bears: first arg must be low series"),
        };
        let close = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bears: second arg must be close series"),
        };

        let period = match &args[2] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("Bears: third arg must be scalar period"),
        };

        let ema = MovingAverage { method: MAMethod::Exponential };
        let ema_val = ema.execute(&[close, dsl::lit(period)])?;

        Ok(low - ema_val)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Bears Power is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iBearsPower(_Symbol, _Period, {}, PRICE_CLOSE)", args[2])
    }
}

// --- Bulls Power ---
pub struct Bulls {
    pub period: usize,
}

impl Indicator for Bulls {
    fn alias(&self) -> &'static str { "Bulls" }
    fn ui_name(&self) -> &'static str { "Bulls Power" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 3 } // high, close, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // close
            DataType::Integer,       // period
        ]
    }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let high = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bulls: first arg must be high series"),
        };
        let close = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Bulls: second arg must be close series"),
        };

        let period = match &args[2] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("Bulls: third arg must be scalar period"),
        };

        let ema = MovingAverage { method: MAMethod::Exponential };
        let ema_val = ema.execute(&[close, dsl::lit(period)])?;

        Ok(high - ema_val)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Bulls Power is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iBullsPower(_Symbol, _Period, {}, PRICE_CLOSE)", args[2])
    }
}

// --- DEMA (Double Exponential Moving Average) ---
pub struct DEMA {
    pub period: usize,
}


impl Indicator for DEMA {
    fn alias(&self) -> &'static str { "DEMA" }
    fn ui_name(&self) -> &'static str { "Double Exponential Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
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
            _ => bail!("DEMA: first arg must be close series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("DEMA: second arg must be scalar period"),
        };

        let ema1 = MovingAverage { method: MAMethod::Exponential };
        let ema1_val = ema1.execute(&[close, dsl::lit(period)])?;

        let ema2 = MovingAverage { method: MAMethod::Exponential };
        let ema2_val = ema2.execute(&[ema1_val.clone(), dsl::lit(period)])?;

        Ok(dsl::lit(2.0) * ema1_val - ema2_val)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("DEMA is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMAOnArray(DEMA_buffer, 0, {}, 0, MODE_EMA, 0)", self.period)
    }
}

// --- TEMA (Triple Exponential Moving Average) ---
pub struct TEMA {
    pub period: usize,
}


impl Indicator for TEMA {
    fn alias(&self) -> &'static str { "TEMA" }
    fn ui_name(&self) -> &'static str { "Triple Exponential Moving Average" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
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
            _ => bail!("TEMA: first arg must be close series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("TEMA: second arg must be scalar period"),
        };

        let ema1 = MovingAverage { method: MAMethod::Exponential };
        let ema1_val = ema1.execute(&[close, dsl::lit(period)])?;

        let ema2 = MovingAverage { method: MAMethod::Exponential };
        let ema2_val = ema2.execute(&[ema1_val.clone(), dsl::lit(period)])?;

        let ema3 = MovingAverage { method: MAMethod::Exponential };
        let ema3_val = ema3.execute(&[ema2_val.clone(), dsl::lit(period)])?;

        Ok(dsl::lit(3.0) * (ema1_val - ema2_val) + ema3_val)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("TEMA is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMAOnArray(TEMA_buffer, 0, {}, 0, MODE_EMA, 0)", self.period)
    }
}

// --- TriX (Triple Exponential Average) ---
pub struct TriX {
    pub period: usize,
}


impl Indicator for TriX {
    fn alias(&self) -> &'static str { "TriX" }
    fn ui_name(&self) -> &'static str { "Triple Exponential Average" }
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
            _ => bail!("TriX: first arg must be close series"),
        };
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as i64,
            _ => bail!("TriX: second arg must be scalar period"),
        };

        let ema1 = MovingAverage { method: MAMethod::Exponential };
        let ema1_val = ema1.execute(&[close, dsl::lit(period)])?;

        let ema2 = MovingAverage { method: MAMethod::Exponential };
        let ema2_val = ema2.execute(&[ema1_val, dsl::lit(period)])?;

        let ema3 = MovingAverage { method: MAMethod::Exponential };
        let ema3_val = ema3.execute(&[ema2_val, dsl::lit(period)])?;

        let prev_ema3 = ema3_val.clone().shift(dsl::lit(1));
        Ok(((ema3_val - prev_ema3.clone()) / prev_ema3) * dsl::lit(100.0))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("TriX is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iTriX(_Symbol, _Period, {})", self.period)
    }
}