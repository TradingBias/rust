use std::any::Any;
use anyhow::{Result, bail};
use polars::lazy::dsl;
use polars::prelude::{col, when, EWMOptions};
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};

// --- OBV (On-Balance Volume) ---
pub struct OBV;


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

        let prev_close = close.shift(dsl::lit(1));
        let signed_volume = when(close.clone().gt(prev_close.clone()))
            .then(volume.clone())
            .when(close.lt(prev_close))
            .then(-volume)
            .otherwise(dsl::lit(0.0));

        Ok(signed_volume.cum_sum(false))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("OBV is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iOBV(_Symbol, _Period)".to_string()
    }
}

// --- MFI (Money Flow Index) ---
pub struct MFI {
    pub period: usize,
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

        let period = match &args[4] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("MFI: fifth arg must be scalar period"),
        };

        let typical_price = (high + low + close.clone()) / dsl::lit(3.0);
        let prev_typical_price = typical_price.clone().shift(dsl::lit(1));

        let raw_money_flow = typical_price.clone() * volume;

        let positive_money_flow = when(typical_price.clone().gt(prev_typical_price.clone()))
            .then(raw_money_flow.clone())
            .otherwise(dsl::lit(0.0));

        let negative_money_flow = when(typical_price.lt(prev_typical_price))
            .then(raw_money_flow)
            .otherwise(dsl::lit(0.0));

        let positive_mf_sum = positive_money_flow.rolling_sum(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        let negative_mf_sum = negative_money_flow.rolling_sum(polars::prelude::RollingOptions {
            window_size: polars::prelude::Duration::new(period as i64),
            ..Default::default()
        });

        let money_ratio = positive_mf_sum.clone() / negative_mf_sum;
        let money_flow_index = dsl::lit(100.0) - (dsl::lit(100.0) / (dsl::lit(1.0) + money_ratio));

        Ok(money_flow_index)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("MFI is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iMFI(_Symbol, _Period, {})", self.period)
    }
}

// --- Force Index ---
pub struct Force {
    pub period: usize,
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

        let period = match &args[2] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Force: third arg must be scalar period"),
        };

        let force = (close.clone() - close.shift(dsl::lit(1))) * volume;

        Ok(force.ewm_mean(polars::prelude::EWMOptions {
            alpha: 2.0 / (period as f64 + 1.0),
            adjust: false,
            min_periods: period,
            ..Default::default()
        }))
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Force is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
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

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Volumes is a vectorized indicator")
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


impl Indicator for Chaikin {
    fn alias(&self) -> &'static str { "Chaikin" }
    fn ui_name(&self) -> &'static str { "Chaikin Oscillator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Volume }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 6 } // high, low, close, volume, fast_period, slow_period
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries, // high
            DataType::NumericSeries, // low
            DataType::NumericSeries, // close
            DataType::NumericSeries, // volume
            DataType::Integer,       // fast_period
            DataType::Integer,       // slow_period
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
        let fast_period = match &args[4] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Chaikin: fifth arg must be scalar fast_period"),
        };
        let slow_period = match &args[5] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("Chaikin: sixth arg must be scalar slow_period"),
        };

        let adl = money_flow_volume.cum_sum(false);

        let ema_fast = adl.clone().ewm_mean(EWMOptions {
            alpha: 2.0 / (fast_period as f64 + 1.0),
            adjust: false,
            min_periods: fast_period,
            ..Default::default()
        });

        let ema_slow = adl.ewm_mean(EWMOptions {
            alpha: 2.0 / (slow_period as f64 + 1.0),
            adjust: false,
            min_periods: slow_period,
            ..Default::default()
        });

        Ok(ema_fast - ema_slow)
    }

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("Chaikin is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        format!("iChaikin(_Symbol, _Period, {}, {})", self.fast_period, self.slow_period)
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

    fn calculate_stateful(&self, _args: &[f64], _state: &mut dyn Any) -> Result<f64> {
        bail!("BWMFI is a vectorized indicator")
    }

    fn init_state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn generate_mql5(&self, _args: &[String]) -> String {
        "iBWMFI(_Symbol, _Period)".to_string()
    }
}
