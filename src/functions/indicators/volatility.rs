use std::any::Any;
use anyhow::{Result, bail};
use polars::lazy::dsl;
use polars::prelude::{ewm_mean, EWMOptions};
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};

// --- ATR (Average True Range) ---
pub struct ATR {
    pub period: usize,
}

pub struct ATRState {
    period: usize,
    prev_close: Option<f64>,
    true_ranges: Vec<f64>,
    atr: Option<f64>,
}

impl Indicator for ATR {
    fn alias(&self) -> &'static str { "ATR" }
    fn ui_name(&self) -> &'static str { "Average True Range" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volatility }
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
            _ => bail!("ATR: first arg must be high series"),
        };
        let low = match &args[1] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("ATR: second arg must be low series"),
        };
        let close = match &args[2] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("ATR: third arg must be close series"),
        };

        let prev_close = close.shift(1);

        let tr1 = high.clone() - low.clone();
        let tr2 = (high - prev_close.clone()).abs();
        let tr3 = (low - prev_close).abs();

        let true_range = dsl::max(tr1, dsl::max(tr2, tr3));

        let atr = ewm_mean(
            true_range,
            EWMOptions {
                alpha: 1.0 / self.period as f64,
                adjust: false,
                min_periods: self.period,
                ..Default::default()
            },
        );

        Ok(atr)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ATRState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];

        if let Some(prev_close) = state.prev_close {
            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();
            let true_range = tr1.max(tr2).max(tr3);

            state.true_ranges.push(true_range);
            if state.true_ranges.len() > state.period {
                state.true_ranges.remove(0);
            }

            if state.true_ranges.len() == state.period {
                let atr = if let Some(prev_atr) = state.atr {
                    (prev_atr * (state.period - 1) as f64 + true_range) / state.period as f64
                } else {
                    state.true_ranges.iter().sum::<f64>() / state.period as f64
                };
                state.atr = Some(atr);
            }
        }

        state.prev_close = Some(close);
        Ok(state.atr.unwrap_or(0.0))
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ATRState {
            period: self.period,
            prev_close: None,
            true_ranges: Vec::with_capacity(self.period),
            atr: None,
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iATR(_Symbol, _Period, {})", self.period)
    }
}

// --- ADX (Average Directional Index) ---
pub struct ADX {
    pub period: usize,
}

pub struct ADXState {
    period: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    p_dm_buffer: VecDeque<f64>,
    m_dm_buffer: VecDeque<f64>,
    tr_buffer: VecDeque<f64>,
    adx_buffer: VecDeque<f64>,
    p_dm_smooth: f64,
    m_dm_smooth: f64,
    tr_smooth: f64,
}

impl Indicator for ADX {
    fn alias(&self) -> &'static str { "ADX" }
    fn ui_name(&self) -> &'static str { "Average Directional Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
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
        crate::functions::traits::CalculationMode::Stateful
    }

    fn calculate_vectorized(&self, _args: &[IndicatorArg]) -> Result<dsl::Expr> {
        bail!("ADX is a stateful indicator and does not have a vectorized implementation.")
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ADXState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];

        if let (Some(prev_high), Some(prev_low), Some(prev_close)) = (state.prev_high, state.prev_low, state.prev_close) {
            let up_move = high - prev_high;
            let down_move = prev_low - low;

            let p_dm = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
            let m_dm = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };

            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();
            let tr = tr1.max(tr2).max(tr3);

            state.p_dm_buffer.push_back(p_dm);
            state.m_dm_buffer.push_back(m_dm);
            state.tr_buffer.push_back(tr);

            if state.p_dm_buffer.len() > state.period {
                state.p_dm_buffer.pop_front();
                state.m_dm_buffer.pop_front();
                state.tr_buffer.pop_front();
            }

            if state.p_dm_buffer.len() == state.period {
                 state.p_dm_smooth = (state.p_dm_smooth * (state.period - 1) as f64 + p_dm) / state.period as f64;
                 state.m_dm_smooth = (state.m_dm_smooth * (state.period - 1) as f64 + m_dm) / state.period as f64;
                 state.tr_smooth = (state.tr_smooth * (state.period - 1) as f64 + tr) / state.period as f64;

                let p_di = 100.0 * state.p_dm_smooth / state.tr_smooth;
                let m_di = 100.0 * state.m_dm_smooth / state.tr_smooth;

                let dx = 100.0 * (p_di - m_di).abs() / (p_di + m_di);
                state.adx_buffer.push_back(dx);

                if state.adx_buffer.len() > state.period {
                    state.adx_buffer.pop_front();
                }

                if state.adx_buffer.len() == state.period {
                    let adx: f64 = state.adx_buffer.iter().sum::<f64>() / state.period as f64;
                    return Ok(adx);
                }
            }
        }

        state.prev_high = Some(high);
        state.prev_low = Some(low);
        state.prev_close = Some(close);
        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ADXState {
            period: self.period,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            p_dm_buffer: VecDeque::with_capacity(self.period),
            m_dm_buffer: VecDeque::with_capacity(self.period),
            tr_buffer: VecDeque::with_capacity(self.period),
            adx_buffer: VecDeque::with_capacity(self.period),
            p_dm_smooth: 0.0,
            m_dm_smooth: 0.0,
            tr_smooth: 0.0,
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iADX(_Symbol, _Period, {})", self.period)
    }
}

// --- StdDev (Standard Deviation) ---
pub struct StdDev {
    pub period: usize,
}

pub struct StdDevState {
    period: usize,
    buffer: VecDeque<f64>,
}

impl Indicator for StdDev {
    fn alias(&self) -> &'static str { "StdDev" }
    fn ui_name(&self) -> &'static str { "Standard Deviation" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volatility }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 2 } // series, period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // series
            DataType::Integer,       // period
        ]
    }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("StdDev: first arg must be a series"),
        };

        Ok(series.rolling_std(polars::prelude::RollingOptionsFixedWindow {
            window_size: self.period,
            ..Default::default()
        }))
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<StdDevState>().unwrap();
        let value = args[0];
        state.buffer.push_back(value);
        if state.buffer.len() > state.period {
            state.buffer.pop_front();
        }

        if state.buffer.len() < state.period {
            return Ok(0.0);
        }

        let sum: f64 = state.buffer.iter().sum();
        let mean = sum / state.period as f64;
        let variance: f64 = state.buffer.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / state.period as f64;

        Ok(variance.sqrt())
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(StdDevState {
            period: self.period,
            buffer: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iStdDev(_Symbol, _Period, {}, 0, MODE_SMA, PRICE_CLOSE)", self.period)
    }
}
