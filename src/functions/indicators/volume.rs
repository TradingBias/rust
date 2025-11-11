use std::any::Any;
use anyhow::{Result, bail};
use polars::lazy::dsl;
use polars::prelude::{col, cum_sum, when};
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};

// --- OBV (On-Balance Volume) ---
pub struct OBV;

pub struct OBVState {
    prev_close: Option<f64>,
    obv: f64,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<OBVState>().unwrap();
        let close = args[0];
        let volume = args[1];

        if let Some(prev_close) = state.prev_close {
            if close > prev_close {
                state.obv += volume;
            } else if close < prev_close {
                state.obv -= volume;
            }
        }

        state.prev_close = Some(close);
        Ok(state.obv)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(OBVState {
            prev_close: None,
            obv: 0.0,
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iOBV(_Symbol, _Period)".to_string()
    }
}

// --- MFI (Money Flow Index) ---
pub struct MFI {
    pub period: usize,
}

pub struct MFIState {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    closes: VecDeque<f64>,
    volumes: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<MFIState>().unwrap();
        state.highs.push_back(args[0]);
        state.lows.push_back(args[1]);
        state.closes.push_back(args[2]);
        state.volumes.push_back(args[3]);

        if state.highs.len() > state.period {
            state.highs.pop_front();
            state.lows.pop_front();
            state.closes.pop_front();
            state.volumes.pop_front();
        }

        if state.highs.len() == state.period {
            let mut positive_mf = 0.0;
            let mut negative_mf = 0.0;

            for i in 1..state.period {
                let tp_curr = (state.highs[i] + state.lows[i] + state.closes[i]) / 3.0;
                let tp_prev = (state.highs[i-1] + state.lows[i-1] + state.closes[i-1]) / 3.0;
                let money_flow = tp_curr * state.volumes[i];

                if tp_curr > tp_prev {
                    positive_mf += money_flow;
                } else {
                    negative_mf += money_flow;
                }
            }

            if negative_mf == 0.0 {
                return Ok(100.0);
            }

            let money_ratio = positive_mf / negative_mf;
            let mfi = 100.0 - (100.0 / (1.0 + money_ratio));
            return Ok(mfi);
        }

        Ok(50.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(MFIState {
            period: self.period,
            highs: VecDeque::with_capacity(self.period),
            lows: VecDeque::with_capacity(self.period),
            closes: VecDeque::with_capacity(self.period),
            volumes: VecDeque::with_capacity(self.period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMFI(_Symbol, _Period, {})", self.period)
    }
}

// --- Force Index ---
pub struct Force {
    pub period: usize,
}

pub struct ForceState {
    period: usize,
    prev_close: Option<f64>,
    ema: Option<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ForceState>().unwrap();
        let close = args[0];
        let volume = args[1];

        if let Some(prev_close) = state.prev_close {
            let force = (close - prev_close) * volume;
            if let Some(ema) = state.ema {
                let alpha = 2.0 / (state.period as f64 + 1.0);
                state.ema = Some(alpha * force + (1.0 - alpha) * ema);
            } else {
                state.ema = Some(force);
            }
        }

        state.prev_close = Some(close);
        Ok(state.ema.unwrap_or(0.0))
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ForceState {
            period: self.period,
            prev_close: None,
            ema: None,
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iForce(_Symbol, _Period, {}, MODE_EMA, PRICE_CLOSE)", self.period)
    }
}

// --- Volumes ---
pub struct Volumes;

impl Indicator for Volumes {
    fn alias(&self) -> &'static str { "Volumes" }
    fn ui_name(&self) -> &'static str { "Volumes" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 1 } // volume
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries]
    }

    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<dsl::Expr> {
        let volume = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("Volumes: first arg must be volume series"),
        };
        Ok(volume)
    }

    fn calculate_stateful(&self, args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        Ok(args[0])
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iVolumes(_Symbol, _Period)".to_string()
    }
}

// --- Chaikin Oscillator ---
pub struct Chaikin {
    pub fast_period: usize,
    pub slow_period: usize,
}

pub struct ChaikinState {
    fast_period: usize,
    slow_period: usize,
    adl_buffer: VecDeque<f64>,
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

    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<ChaikinState>().unwrap();
        let high = args[0];
        let low = args[1];
        let close = args[2];
        let volume = args[3];

        let mfm = if high == low { 0.0 } else { ((close - low) - (high - close)) / (high - low) };
        let mfv = mfm * volume;
        let prev_adl = state.adl_buffer.back().cloned().unwrap_or(0.0);
        let adl = prev_adl + mfv;
        state.adl_buffer.push_back(adl);

        if state.adl_buffer.len() > state.slow_period {
            state.adl_buffer.pop_front();
        }

        if state.adl_buffer.len() >= state.fast_period {
            let ema_fast = self.calculate_ema(&state.adl_buffer, state.fast_period);
            if state.adl_buffer.len() >= state.slow_period {
                let ema_slow = self.calculate_ema(&state.adl_buffer, state.slow_period);
                return Ok(ema_fast - ema_slow);
            }
        }

        Ok(0.0)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(ChaikinState {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            adl_buffer: VecDeque::with_capacity(self.slow_period),
        })
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iAD(_Symbol, _Period)")
    }
}

impl Chaikin {
    fn calculate_ema(&self, data: &VecDeque<f64>, period: usize) -> f64 {
        let mut ema = data[0];
        let alpha = 2.0 / (period as f64 + 1.0);
        for i in 1..data.len() {
            ema = alpha * data[i] + (1.0 - alpha) * ema;
        }
        ema
    }
}

// --- BWMFI (Market Facilitation Index) ---
pub struct BWMFI;

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

    fn calculate_stateful(&self, args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        let high = args[0];
        let low = args[1];
        let volume = args[2];
        if volume == 0.0 {
            return Ok(0.0);
        }
        Ok((high - low) / volume)
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iBWMFI(_Symbol, _Period)".to_string()
    }
}
