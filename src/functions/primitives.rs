use anyhow::{bail, Result};
use polars::prelude::{DataType as PolarsDataType, Duration, EWMOptions, LiteralValue, RollingOptions};
use polars::lazy::dsl;

pub trait Primitive {
    fn name(&self) -> &'static str;
    fn arity(&self) -> usize;
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr>;
}

// --- Moving Average ---
pub enum MAMethod {
    Simple,
    Exponential,
    // ... other MA types
}

pub struct MovingAverage {
    pub method: MAMethod,
}

impl Primitive for MovingAverage {
    fn name(&self) -> &'static str { "MA" }
    fn arity(&self) -> usize { 2 } // series, period
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        let period = match &args[1] {
            dsl::Expr::Literal(LiteralValue::Int64(p)) => *p as usize,
            _ => bail!("MA period must be an integer literal"),
        };

        match self.method {
            MAMethod::Simple => {
                let options = RollingOptions {
                    window_size: Duration::parse(&format!("{}i", period)),
                    min_periods: period,
                    ..Default::default()
                };
                Ok(series.rolling_mean(options))
            },
            MAMethod::Exponential => {
                let options = EWMOptions {
                    alpha: 2.0 / (period as f64 + 1.0),
                    adjust: false,
                    min_periods: period,
                    ..Default::default()
                };
                Ok(series.ewm_mean(options))
            }
        }
    }
}

// --- Standard Deviation ---
pub struct StdDev;

impl Primitive for StdDev {
    fn name(&self) -> &'static str { "StdDev" }
    fn arity(&self) -> usize { 2 } // series, period
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        let period = match &args[1] {
            dsl::Expr::Literal(LiteralValue::Int64(p)) => *p as usize,
            _ => bail!("StdDev period must be an integer literal"),
        };
        let options = RollingOptions {
            window_size: Duration::parse(&format!("{}i", period)),
            min_periods: period,
            ..Default::default()
        };
        Ok(series.rolling_std(options))
    }
}

// --- Logical operators ---
pub struct And;
impl Primitive for And {
    fn name(&self) -> &'static str { "And" }
    fn arity(&self) -> usize { 2 } // bool_series, bool_series
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().and(args[1].clone()))
    }
}

pub struct Or;
impl Primitive for Or {
    fn name(&self) -> &'static str { "Or" }
    fn arity(&self) -> usize { 2 } // bool_series, bool_series
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().or(args[1].clone()))
    }
}
pub struct Abs;
impl Primitive for Abs {
    fn name(&self) -> &'static str {
        "Abs"
    }
    fn arity(&self) -> usize {
        1
    }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().abs())
    }
}
