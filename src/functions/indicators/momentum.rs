use std::collections::VecDeque;
use std::any::Any;
use anyhow::{Result, bail};
use polars::prelude::{EWMOptions};
use polars::lazy::dsl;
use polars::series::ops::NullBehavior;
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};
use crate::functions::indicators::trend::{SMA, SMAState};

// --- AC (Accelerator Oscillator) ---
pub struct AC;

pub struct AOState {
    slow_sma_state: SMAState,
    fast_sma_state: SMAState,
    slow_sma_indicator: SMA,
    fast_sma_indicator: SMA,
}

pub struct ACState {
    ao_state: AOState,
    ao_indicator: AO,
    ao_buffer: VecDeque<f64>,
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

/// State for stateful RSI calculation
pub struct RSIState {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    prev_close: Option<f64>,
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
        
        if state.gains.len() > state.period {
            state.gains.pop_front();
            state.losses.pop_front();
        }
        
        if state.gains.len() < state.period {
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

pub struct StochasticState {
    k_period: usize,
    d_period: usize,
    slowing: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    closes: VecDeque<f64>,
    k_values: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<StochasticState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];

        state.highs.push_back(high);
        state.lows.push_back(low);
        state.closes.push_back(close);

        if state.highs.len() > state.k_period {
            state.highs.pop_front();
            state.lows.pop_front();
            state.closes.pop_front();
        }

        if state.highs.len() == state.k_period {
            let highest_high = state.highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let lowest_low = state.lows.iter().cloned().fold(f64::INFINITY, f64::min);

            let percent_k = if highest_high == lowest_low {
                0.0
            } else {
                (close - lowest_low) / (highest_high - lowest_low) * 100.0
            };

            state.k_values.push_back(percent_k);
            if state.k_values.len() > state.d_period {
                state.k_values.pop_front();
            }

            if state.k_values.len() == state.d_period {
                let sum_k: f64 = state.k_values.iter().sum();
                let percent_d = sum_k / state.d_period as f64;
                return Ok(percent_d);
            }
        }
        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(StochasticState {
            k_period: self.k_period,
            d_period: self.d_period,
            slowing: self.slowing,
            highs: VecDeque::with_capacity(self.k_period),
            lows: VecDeque::with_capacity(self.k_period),
            closes: VecDeque::with_capacity(self.k_period),
            k_values: VecDeque::with_capacity(self.d_period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iStochastic(_Symbol, _Period, {}, {}, {}, MODE_SMA, STO_LOWHIGH)", self.k_period, self.d_period, self.slowing)
    }
}

// --- CCI (Commodity Channel Index) ---
pub struct CCI {
    pub period: usize,
}

pub struct CCIState {
    period: usize,
    tp_buffer: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<CCIState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];

        let typical_price = (high + low + close) / 3.0;
        state.tp_buffer.push_back(typical_price);

        if state.tp_buffer.len() > state.period {
            state.tp_buffer.pop_front();
        }

        if state.tp_buffer.len() == state.period {
            let sum_tp: f64 = state.tp_buffer.iter().sum();
            let sma_tp = sum_tp / state.period as f64;

            let mean_deviation: f64 = state.tp_buffer.iter().map(|tp| (tp - sma_tp).abs()).sum::<f64>() / state.period as f64;

            if mean_deviation == 0.0 {
                return Ok(0.0);
            }

            let cci = (typical_price - sma_tp) / (0.015 * mean_deviation);
            return Ok(cci);
        }
        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(CCIState {
            period: self.period,
            tp_buffer: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iCCI(_Symbol, _Period, {}, PRICE_TYPICAL)", self.period)
    }
}

// --- Williams' %R ---
pub struct WilliamsR {
    pub period: usize,
}

pub struct WilliamsRState {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<WilliamsRState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];

        state.highs.push_back(high);
        state.lows.push_back(low);

        if state.highs.len() > state.period {
            state.highs.pop_front();
            state.lows.pop_front();
        }

        if state.highs.len() == state.period {
            let highest_high = state.highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let lowest_low = state.lows.iter().cloned().fold(f64::INFINITY, f64::min);
            if highest_high == lowest_low {
                return Ok(0.0);
            }
            return Ok(((highest_high - close) / (highest_high - lowest_low)) * -100.0);
        }

        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(WilliamsRState {
            period: self.period,
            highs: VecDeque::with_capacity(self.period),
            lows: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iWPR(_Symbol, _Period, {})", self.period)
    }
}

// --- ROC (Rate of Change) ---
pub struct ROC {
    pub period: usize,
}

pub struct ROCState {
    period: usize,
    prices: VecDeque<f64>,
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

        let prev_close = close.shift(self.period);

        Ok(((close - prev_close.clone()) / prev_close) * dsl::lit(100.0))
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ROCState>().unwrap();
        let close = args[0];
        state.prices.push_back(close);
        if state.prices.len() > state.period {
            state.prices.pop_front();
        }

        if state.prices.len() == state.period {
            let prev_price = state.prices[0];
            if prev_price == 0.0 {
                return Ok(0.0);
            }
            return Ok(((close - prev_price) / prev_price) * 100.0);
        }

        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ROCState {
            period: self.period,
            prices: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}

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
        let ao = median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 34, ..Default::default() });

        let ac = ao.clone() - ao.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() });
        Ok(ac)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ACState>().unwrap();
        let high = args[0];
        let low = args[1];

        let median_price = (high + low) / 2.0;
        let ao = state.ao_indicator.calculate_stateful(&[high, low], &mut state.ao_state)?;

        state.ao_buffer.push_back(ao);
        if state.ao_buffer.len() > 5 {
            state.ao_buffer.pop_front();
        }

        if state.ao_buffer.len() < 5 {
            return Ok(0.0);
        }

        let sma_ao: f64 = state.ao_buffer.iter().sum::<f64>() / 5.0;

        Ok(ao - sma_ao)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ACState {
            ao_state: AOState {
                slow_sma_state: SMAState { period: 34, buffer: VecDeque::with_capacity(34) },
                fast_sma_state: SMAState { period: 5, buffer: VecDeque::with_capacity(5) },
                slow_sma_indicator: SMA { period: 34 },
                fast_sma_indicator: SMA { period: 5 },
            },
            ao_indicator: AO,
            ao_buffer: VecDeque::with_capacity(5),
        })
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
        let ao = median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 5, ..Default::default() }) -
                 median_price.rolling_mean(polars::prelude::RollingOptionsFixedWindow { window_size: 34, ..Default::default() });

        Ok(ao)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<AOState>().unwrap();
        let high = args[0];
        let low = args[1];

        let median_price = (high + low) / 2.0;

        let slow_sma = state.slow_sma_indicator.calculate_stateful(&[median_price], &mut state.slow_sma_state)?;
        let fast_sma = state.fast_sma_indicator.calculate_stateful(&[median_price], &mut state.fast_sma_state)?;

        Ok(fast_sma - slow_sma)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(AOState {
            slow_sma_state: SMAState { period: 34, buffer: VecDeque::with_capacity(34) },
            fast_sma_state: SMAState { period: 5, buffer: VecDeque::with_capacity(5) },
            slow_sma_indicator: SMA { period: 34 },
            fast_sma_indicator: SMA { period: 5 },
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iAO(_Symbol, _Period)".to_string()
    }
}

// --- RVI (Relative Vigor Index) ---
pub struct RVI {
    pub period: usize,
}

pub struct RVIState {
    period: usize,
    opens: VecDeque<f64>,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    closes: VecDeque<f64>,
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

        let numerator = (close.clone() - open.clone()) + 2.0 * (close.shift(1) - open.shift(1)) + 2.0 * (close.shift(2) - open.shift(2)) + (close.shift(3) - open.shift(3));
        let denominator = (high.clone() - low.clone()) + 2.0 * (high.shift(1) - low.shift(1)) + 2.0 * (high.shift(2) - low.shift(2)) + (high.shift(3) - low.shift(3));

        let rvi = numerator.rolling_sum(polars::prelude::RollingOptionsFixedWindow { window_size: self.period, ..Default::default() }) /
                  denominator.rolling_sum(polars::prelude::RollingOptionsFixedWindow { window_size: self.period, ..Default::default() });

        Ok(rvi)
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<RVIState>().unwrap();
        state.opens.push_back(args[0]);
        state.highs.push_back(args[1]);
        state.lows.push_back(args[2]);
        state.closes.push_back(args[3]);

        if state.opens.len() > self.period {
            state.opens.pop_front();
            state.highs.pop_front();
            state.lows.pop_front();
            state.closes.pop_front();
        }

        if state.opens.len() == self.period {
            let mut num_sum = 0.0;
            let mut den_sum = 0.0;
            for i in 3..self.period {
                num_sum += (state.closes[i] - state.opens[i]) + 2.0 * (state.closes[i-1] - state.opens[i-1]) + 2.0 * (state.closes[i-2] - state.opens[i-2]) + (state.closes[i-3] - state.opens[i-3]);
                den_sum += (state.highs[i] - state.lows[i]) + 2.0 * (state.highs[i-1] - state.lows[i-1]) + 2.0 * (state.highs[i-2] - state.lows[i-2]) + (state.highs[i-3] - state.lows[i-3]);
            }
            if den_sum == 0.0 {
                return Ok(0.0);
            }
            return Ok(num_sum / den_sum);
        }

        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(RVIState {
            period: self.period,
            opens: VecDeque::with_capacity(self.period),
            highs: VecDeque::with_capacity(self.period),
            lows: VecDeque::with_capacity(self.period),
            closes: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iRVI(_Symbol, _Period, {})", self.period)
    }
}

// --- DeMarker ---
pub struct DeMarker {
    pub period: usize,
}

pub struct DeMarkerState {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<DeMarkerState>().unwrap();
        let high = args[0];
        let low = args[1];

        state.highs.push_back(high);
        state.lows.push_back(low);

        if state.highs.len() > self.period {
            state.highs.pop_front();
            state.lows.pop_front();
        }

        if state.highs.len() == self.period {
            let mut de_max_sum = 0.0;
            let mut de_min_sum = 0.0;

            for i in 1..self.period {
                if state.highs[i] > state.highs[i-1] {
                    de_max_sum += state.highs[i] - state.highs[i-1];
                }
                if state.lows[i] < state.lows[i-1] {
                    de_min_sum += state.lows[i-1] - state.lows[i];
                }
            }

            let sma_de_max = de_max_sum / self.period as f64;
            let sma_de_min = de_min_sum / self.period as f64;

            if sma_de_max + sma_de_min == 0.0 {
                return Ok(0.5);
            }

            return Ok(sma_de_max / (sma_de_max + sma_de_min));
        }

        Ok(0.5)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(DeMarkerState {
            period: self.period,
            highs: VecDeque::with_capacity(self.period),
            lows: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iDeMarker(_Symbol, _Period, {})", self.period)
    }
}

// --- Momentum ---
pub struct Momentum {
    pub period: usize,
}

pub struct MomentumState {
    period: usize,
    prices: VecDeque<f64>,
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

        Ok(close.clone() - close.shift(self.period))
    }

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<MomentumState>().unwrap();
        let close = args[0];
        state.prices.push_back(close);
        if state.prices.len() > self.period {
            state.prices.pop_front();
        }

        if state.prices.len() == self.period {
            let prev_price = state.prices[0];
            return Ok(close - prev_price);
        }

        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(MomentumState {
            period: self.period,
            prices: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMomentum(_Symbol, _Period, {}, PRICE_CLOSE)", self.period)
    }
}