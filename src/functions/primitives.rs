use anyhow::{bail, Result};
use polars::prelude::{EWMOptions, LiteralValue, RollingOptionsFixedWindow};
use polars::lazy::dsl::{self};
use crate::functions::traits::Primitive;
use polars::datatypes::AnyValue;



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
    fn ui_name(&self) -> &'static str { "Moving Average" }
    fn alias(&self) -> &'static str { "MA" }
    fn arity(&self) -> usize { 2 } // series, period
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Integer]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        let period = match &args[1] {
            dsl::Expr::Literal(LiteralValue::Scalar(p)) => {
                if let AnyValue::Int64(val) = p.to_owned().value() {
                    *val as usize
                } else {
                    bail!("MA period must be an integer literal")
                }
            },
            _ => bail!("MA period must be an integer literal"),
        };

        match self.method {
            MAMethod::Simple => {
                let options = RollingOptionsFixedWindow {
                    window_size: period as usize,
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
    fn generate_mql5(&self, args: &[String]) -> String {
        let period_str = match &args[1] {
            s if s.parse::<i64>().is_ok() => s.clone(),
            _ => "0".to_string(), // Default or placeholder if not a direct literal
        };
        format!("iMA({}, {}, {}, 0, MODE_SMA, {}, {})", args[0], args[1], period_str, args[2], args[3])
    }
}

// --- Standard Deviation ---
pub struct StdDev;

impl Primitive for StdDev {
    fn ui_name(&self) -> &'static str { "Standard Deviation" }
    fn alias(&self) -> &'static str { "StdDev" }
    fn arity(&self) -> usize { 2 } // series, period
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Integer]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        let period = match &args[1] {
            dsl::Expr::Literal(LiteralValue::Scalar(p)) => {
                if let AnyValue::Int64(val) = p.to_owned().value() {
                    *val as usize
                } else {
                    bail!("StdDev period must be an integer literal")
                }
            },
            _ => bail!("StdDev period must be an integer literal"),
        };
        let options = RollingOptionsFixedWindow {
            window_size: period as usize,
            min_periods: period,
            ..Default::default()
        };
        Ok(series.rolling_std(options))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        let period_str = match &args[1] {
            s if s.parse::<i64>().is_ok() => s.clone(),
            _ => "0".to_string(), // Default or placeholder if not a direct literal
        };
        format!("iStdDev({}, {}, {}, 0, MODE_SMA, PRICE_CLOSE)", args[0], args[1], period_str)
    }
}

// --- Logical operators ---
pub struct And;
impl Primitive for And {
    fn ui_name(&self) -> &'static str { "Logical AND" }
    fn alias(&self) -> &'static str { "And" }
    fn arity(&self) -> usize { 2 } // bool_series, bool_series
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::BoolSeries, crate::types::DataType::BoolSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().and(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} && {})", args[0], args[1])
    }
}

pub struct Or;
impl Primitive for Or {
    fn ui_name(&self) -> &'static str { "Logical OR" }
    fn alias(&self) -> &'static str { "Or" }
    fn arity(&self) -> usize { 2 } // bool_series, bool_series
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::BoolSeries, crate::types::DataType::BoolSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().or(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} || {})", args[0], args[1])
    }
}
pub struct Abs;
impl Primitive for Abs {
    fn ui_name(&self) -> &'static str { "Absolute Value" }
    fn alias(&self) -> &'static str { "Abs" }
    fn arity(&self) -> usize { 1 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().abs())
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("MathAbs({})", args[0])
    }
}
