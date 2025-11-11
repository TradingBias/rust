use anyhow::{bail, Result};
use polars::prelude::{DataType as PolarsDataType, LiteralValue, RollingOptions, EWMOptions, Duration};
use polars::lazy::dsl;
use crate::functions::traits::Primitive;

// Helper to extract integer literals from expressions
fn extract_literal_int(expr: &dsl::Expr) -> Result<i64> {
    match expr {
        dsl::Expr::Literal(lv) => match lv {
            LiteralValue::Int8(v) => Ok(*v as i64),
            LiteralValue::Int16(v) => Ok(*v as i64),
            LiteralValue::Int32(v) => Ok(*v as i64),
            LiteralValue::Int64(v) => Ok(*v),
            _ => bail!("Literal is not a convertible integer: {:?}", lv),
        },
        _ => bail!("Failed to extract literal integer from expression: {:?}", expr),
    }
}

// --- Primitive Implementations ---

#[derive(Debug, Clone, Copy)]
pub enum MAMethod {
    Simple,
    Exponential,
    Linear,
    Smoothed,
}

pub struct MovingAverage {
    pub method: MAMethod,
}

impl Primitive for MovingAverage {
    fn alias(&self) -> &'static str {
        match self.method {
            MAMethod::Simple => "SMA",
            MAMethod::Exponential => "EMA",
            MAMethod::Linear => "LWMA",
            MAMethod::Smoothed => "SMMA",
        }
    }

    fn ui_name(&self) -> &'static str {
        match self.method {
            MAMethod::Simple => "Simple Moving Average",
            MAMethod::Exponential => "Exponential Moving Average",
            MAMethod::Linear => "Linear-Weighted Moving Average",
            MAMethod::Smoothed => "Smoothed Moving Average",
        }
    }

    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] } // Simplified
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }

    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        if args.len() != 2 {
            bail!("MovingAverage requires 2 args: series, period");
        }

        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        let rolling_options = RollingOptions {
            window_size: Duration::new(period),
            min_periods: period as usize,
            ..Default::default()
        };

        match self.method {
            MAMethod::Simple => Ok(series.clone().rolling_mean(rolling_options)),
            MAMethod::Exponential => {
                let ewm_options = EWMOptions {
                    alpha: 2.0 / (period as f64 + 1.0),
                    adjust: false,
                    min_periods: period as usize,
                    ..Default::default()
                };
                Ok(series.clone().ewm_mean(ewm_options))
            }
            MAMethod::Linear => {
                // TODO: Implement proper LWMA for lazy API. Using SMA as placeholder.
                Ok(series.clone().rolling_mean(rolling_options))
            }
            MAMethod::Smoothed => {
                // SMMA is equivalent to an EMA with alpha = 1/period
                let ewm_options = EWMOptions {
                    alpha: 1.0 / period as f64,
                    adjust: false,
                    min_periods: period as usize,
                    ..Default::default()
                };
                Ok(series.clone().ewm_mean(ewm_options))
            }
        }
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        let method_str = match self.method {
            MAMethod::Simple => "MODE_SMA",
            MAMethod::Exponential => "MODE_EMA",
            MAMethod::Linear => "MODE_LWMA",
            MAMethod::Smoothed => "MODE_SMMA",
        };
        format!("TB_MA({}, {}, {})", args[0], args[1], method_str)
    }
}

pub struct Highest;
impl Primitive for Highest {
    fn alias(&self) -> &'static str { "Highest" }
    fn ui_name(&self) -> &'static str { "Highest High" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        let options = RollingOptions { window_size: Duration::new(period), min_periods: period as usize, ..Default::default() };
        Ok(series.clone().rolling_max(options))
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("iHighest({}, {}, {}, {})", args[0], args[1], args[2], args[3]) }
}

pub struct Lowest;
impl Primitive for Lowest {
    fn alias(&self) -> &'static str { "Lowest" }
    fn ui_name(&self) -> &'static str { "Lowest Low" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        let options = RollingOptions { window_size: Duration::new(period), min_periods: period as usize, ..Default::default() };
        Ok(series.clone().rolling_min(options))
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("iLowest({}, {}, {}, {})", args[0], args[1], args[2], args[3]) }
}

pub struct Sum;
impl Primitive for Sum {
    fn alias(&self) -> &'static str { "Sum" }
    fn ui_name(&self) -> &'static str { "Summation" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        let options = RollingOptions { window_size: Duration::new(period), min_periods: period as usize, ..Default::default() };
        Ok(series.clone().rolling_sum(options))
    }
    fn generate_mql5(&self, _args: &[String]) -> String { "// MQL5 for Sum not implemented".to_string() }
}

pub struct StdDev;
impl Primitive for StdDev {
    fn alias(&self) -> &'static str { "StdDev" }
    fn ui_name(&self) -> &'static str { "Standard Deviation" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        let options = RollingOptions { window_size: Duration::new(period), min_periods: period as usize, ..Default::default() };
        Ok(series.clone().rolling_std(options))
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("iStdDev({}, {}, {}, 0, {}, {}, {})", args[0], args[1], args[2], args[3], args[4], args[5]) }
}

pub struct Momentum;
impl Primitive for Momentum {
    fn alias(&self) -> &'static str { "Momentum" }
    fn ui_name(&self) -> &'static str { "Momentum" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        Ok(series.clone() - series.clone().shift(dsl::lit(period)))
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("iMomentum({}, {}, {}, {}, {})", args[0], args[1], args[2], args[3], args[4]) }
}

pub struct Shift;
impl Primitive for Shift {
    fn alias(&self) -> &'static str { "Shift" }
    fn ui_name(&self) -> &'static str { "Shift" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = &args[0];
        let offset = extract_literal_int(&args[1])?;
        Ok(series.clone().shift(dsl::lit(offset)))
    }
    fn generate_mql5(&self, _args: &[String]) -> String { "// MQL5 for Shift not implemented".to_string() }
}

pub struct Absolute;
impl Primitive for Absolute {
    fn alias(&self) -> &'static str { "Absolute" }
    fn ui_name(&self) -> &'static str { "Absolute Value" }
    fn arity(&self) -> usize { 1 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        Ok(dsl::when(series.clone().lt(dsl::lit(0.0))).then(series * dsl::lit(-1.0)).otherwise(series))
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("MathAbs({})", args[0]) }
}

pub struct Divide;
impl Primitive for Divide {
    fn alias(&self) -> &'static str { "Divide" }
    fn ui_name(&self) -> &'static str { "Divide" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() / args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("({} / {})", args[0], args[1]) }
}

pub struct Multiply;
impl Primitive for Multiply {
    fn alias(&self) -> &'static str { "Multiply" }
    fn ui_name(&self) -> &'static str { "Multiply" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() * args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("({} * {})", args[0], args[1]) }
}

pub struct Add;
impl Primitive for Add {
    fn alias(&self) -> &'static str { "Add" }
    fn ui_name(&self) -> &'static str { "Add" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() + args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("({} + {})", args[0], args[1]) }
}

pub struct Subtract;
impl Primitive for Subtract {
    fn alias(&self) -> &'static str { "Subtract" }
    fn ui_name(&self) -> &'static str { "Subtract" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> { vec![] }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() - args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String { format!("({} - {})", args[0], args[1]) }
}