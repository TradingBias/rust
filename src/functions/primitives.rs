use anyhow::{bail, Result};
use polars::prelude::{EWMOptions, LiteralValue, RollingOptionsFixedWindow};
use polars::lazy::dsl::{self};
use crate::functions::traits::Primitive;
use polars::datatypes::AnyValue;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
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
    fn ui_name(&self) -> &'static str { "Moving Average" }
    fn alias(&self) -> &'static str { "MA" }
    fn arity(&self) -> usize { 2 } // series, period
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Integer]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series = args[0].clone();
        // Try to extract the period value from the expression
        let period: usize = match &args[1] {
            dsl::Expr::Literal(lit_val) => {
                // Handle LiteralValue by extracting scalar value
                match lit_val {
                    LiteralValue::Scalar(p) => {
                        // Extract numeric value from AnyValue
                        let owned_p = p.to_owned();
                        let scalar_val = owned_p.value();
                        match scalar_val {
                            AnyValue::Int64(val) => *val as usize,
                            AnyValue::Int32(val) => *val as usize,
                            AnyValue::UInt32(val) => *val as usize,
                            AnyValue::UInt64(val) => *val as usize,
                            AnyValue::Float64(val) => *val as usize,
                            AnyValue::Float32(val) => *val as usize,
                            _ => bail!("MA period must be a numeric literal, got {:?}", scalar_val),
                        }
                    },
                    // For other literal types, try to convert to string and parse
                    other_lit => {
                        // Try to get the debug representation and parse it
                        let debug_str = format!("{:?}", other_lit);
                        // Extract number from patterns like "dyn int: 14"
                        if let Some(num_str) = debug_str.split(": ").nth(1) {
                            if let Ok(val) = num_str.parse::<i64>() {
                                val as usize
                            } else {
                                bail!("MA period must be a numeric literal, got {:?}", other_lit)
                            }
                        } else {
                            bail!("MA period must be a numeric literal, got {:?}", other_lit)
                        }
                    }
                }
            },
            other => bail!("MA period must be an integer literal, got expression type: {:?}", other),
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

// --- Math operators ---

pub struct Add;
impl Primitive for Add {
    fn ui_name(&self) -> &'static str { "Add" }
    fn alias(&self) -> &'static str { "Add" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() + args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} + {})", args[0], args[1])
    }
}

pub struct Subtract;
impl Primitive for Subtract {
    fn ui_name(&self) -> &'static str { "Subtract" }
    fn alias(&self) -> &'static str { "Subtract" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() - args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} - {})", args[0], args[1])
    }
}

pub struct Multiply;
impl Primitive for Multiply {
    fn ui_name(&self) -> &'static str { "Multiply" }
    fn alias(&self) -> &'static str { "Multiply" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() * args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} * {})", args[0], args[1])
    }
}

pub struct Divide;
impl Primitive for Divide {
    fn ui_name(&self) -> &'static str { "Divide" }
    fn alias(&self) -> &'static str { "Divide" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::NumericSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone() / args[1].clone())
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} / {})", args[0], args[1])
    }
}

// --- Comparison operators ---

pub struct GreaterThan;
impl Primitive for GreaterThan {
    fn ui_name(&self) -> &'static str { "Greater Than" }
    fn alias(&self) -> &'static str { "gt" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().gt(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} > {})", args[0], args[1])
    }
}

pub struct LessThan;
impl Primitive for LessThan {
    fn ui_name(&self) -> &'static str { "Less Than" }
    fn alias(&self) -> &'static str { "lt" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().lt(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} < {})", args[0], args[1])
    }
}

pub struct Equal;
impl Primitive for Equal {
    fn ui_name(&self) -> &'static str { "Equal" }
    fn alias(&self) -> &'static str { "eq" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} == {})", args[0], args[1])
    }
}

pub struct NotEqual;
impl Primitive for NotEqual {
    fn ui_name(&self) -> &'static str { "Not Equal" }
    fn alias(&self) -> &'static str { "neq" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().neq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} != {})", args[0], args[1])
    }
}

pub struct GreaterThanOrEqual;
impl Primitive for GreaterThanOrEqual {
    fn ui_name(&self) -> &'static str { "Greater Than or Equal" }
    fn alias(&self) -> &'static str { "gte" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().gt_eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} >= {})", args[0], args[1])
    }
}

pub struct LessThanOrEqual;
impl Primitive for LessThanOrEqual {
    fn ui_name(&self) -> &'static str { "Less Than or Equal" }
    fn alias(&self) -> &'static str { "lte" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().lt_eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} <= {})", args[0], args[1])
    }
}

pub struct GreaterThanScalar;
impl Primitive for GreaterThanScalar {
    fn ui_name(&self) -> &'static str { "Greater Than Scalar" }
    fn alias(&self) -> &'static str { "gt_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().gt(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} > {})", args[0], args[1])
    }
}

pub struct LessThanScalar;
impl Primitive for LessThanScalar {
    fn ui_name(&self) -> &'static str { "Less Than Scalar" }
    fn alias(&self) -> &'static str { "lt_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().lt(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} < {})", args[0], args[1])
    }
}

pub struct EqualScalar;
impl Primitive for EqualScalar {
    fn ui_name(&self) -> &'static str { "Equal Scalar" }
    fn alias(&self) -> &'static str { "eq_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} == {})", args[0], args[1])
    }
}

pub struct NotEqualScalar;
impl Primitive for NotEqualScalar {
    fn ui_name(&self) -> &'static str { "Not Equal Scalar" }
    fn alias(&self) -> &'static str { "neq_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().neq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} != {})", args[0], args[1])
    }
}

pub struct GreaterThanOrEqualScalar;
impl Primitive for GreaterThanOrEqualScalar {
    fn ui_name(&self) -> &'static str { "Greater Than or Equal Scalar" }
    fn alias(&self) -> &'static str { "gte_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().gt_eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} >= {})", args[0], args[1])
    }
}

pub struct LessThanOrEqualScalar;
impl Primitive for LessThanOrEqualScalar {
    fn ui_name(&self) -> &'static str { "Less Than or Equal Scalar" }
    fn alias(&self) -> &'static str { "lte_scalar" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::Float]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        Ok(args[0].clone().lt_eq(args[1].clone()))
    }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("({} <= {})", args[0], args[1])
    }
}

pub struct CrossAbove;
impl Primitive for CrossAbove {
    fn ui_name(&self) -> &'static str { "Cross Above" }
    fn alias(&self) -> &'static str { "cross_above" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series1 = args[0].clone();
        let series2 = args[1].clone();
        let prev_series1 = series1.clone().shift(dsl::lit(1));
        let prev_series2 = series2.clone().shift(dsl::lit(1));

        Ok(series1.gt(series2).and(prev_series1.lt_eq(prev_series2)))
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        // MQL5 doesn't have a direct equivalent, this would need a more complex custom indicator
        "".to_string()
    }
}

pub struct CrossBelow;
impl Primitive for CrossBelow {
    fn ui_name(&self) -> &'static str { "Cross Below" }
    fn alias(&self) -> &'static str { "cross_below" }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<crate::types::DataType> {
        vec![crate::types::DataType::NumericSeries, crate::types::DataType::NumericSeries]
    }
    fn output_type(&self) -> crate::types::DataType { crate::types::DataType::BoolSeries }
    fn execute(&self, args: &[dsl::Expr]) -> Result<dsl::Expr> {
        let series1 = args[0].clone();
        let series2 = args[1].clone();
        let prev_series1 = series1.clone().shift(dsl::lit(1));
        let prev_series2 = series2.clone().shift(dsl::lit(1));

        Ok(series1.lt(series2).and(prev_series1.gt_eq(prev_series2)))
    }
    fn generate_mql5(&self, _args: &[String]) -> String {
        // MQL5 doesn't have a direct equivalent, this would need a more complex custom indicator
        "".to_string()
    }
}
