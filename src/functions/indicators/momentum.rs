use std::collections::VecDeque;
use std::any::Any;
use anyhow::{Result, bail};
use polars::prelude::{EWMOptions};
use polars::lazy::dsl;
use polars::series::ops::NullBehavior;
use crate::functions::traits::{Indicator, IndicatorArg};
use crate::types::{DataType, ScaleType};

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