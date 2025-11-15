use crate::{
    data::IndicatorCache,
    error::{Result, TradebiasError},
    functions::traits::{Indicator, Primitive, IndicatorArg},
    functions::registry::FunctionRegistry,
    types::{AstNode, Value},
};
use polars::prelude::*;
use std::sync::Arc;

pub struct ExpressionBuilder {
    registry: Arc<FunctionRegistry>,
    cache: Arc<IndicatorCache>,
}

impl ExpressionBuilder {
    pub fn new(registry: Arc<FunctionRegistry>, cache: Arc<IndicatorCache>) -> Self {
        Self { registry, cache }
    }

    pub fn build(&self, ast: &AstNode, df: &DataFrame) -> Result<Expr> {
        match ast {
            AstNode::Const(value) => self.build_const(value),
            AstNode::Call { function, args } => self.build_call(function, args, df),
            AstNode::Rule { condition, action } => self.build_rule(condition, action, df),
        }
    }

    fn build_const(&self, value: &Value) -> Result<Expr> {
        Ok(match value {
            Value::Integer(i) => lit(*i),
            Value::Float(f) => lit(*f),
            Value::String(s) => col(s),
            Value::Bool(b) => lit(*b),
        })
    }

    fn build_call(&self, function: &str, args: &[Box<AstNode>], df: &DataFrame) -> Result<Expr> {
        // Handle data accessors (OHLCV columns) as special case
        match function {
            "Open" => return Ok(col("open")),
            "High" => return Ok(col("high")),
            "Low" => return Ok(col("low")),
            "Close" => return Ok(col("close")),
            "Volume" => return Ok(col("volume")),
            _ => {}
        }

        // Note: Caching is disabled because we return expressions directly rather than
        // evaluating them. Caching Series and then converting to lit() causes stack
        // overflow with deeply nested expressions. Expression-level caching would
        // require caching Expr objects, which is not straightforward.
        // Performance impact is minimal for max_depth <= 3.

        if let Some(indicator) = self.registry.get_indicator(function) {
            self.build_indicator_call(indicator.as_ref(), args, df)
        } else if let Some(primitive) = self.registry.get_primitive(function) {
            self.build_primitive_call(primitive.as_ref(), args, df)
        } else {
            Err(TradebiasError::IndicatorError(format!(
                "Function {} not found",
                function
            )))
        }
    }

    fn build_rule(&self, condition: &AstNode, action: &AstNode, df: &DataFrame) -> Result<Expr> {
        let cond_expr = self.build(condition, df)?;
        let action_expr = self.build(action, df)?;
        Ok(when(cond_expr).then(action_expr).otherwise(lit(0.0)))
    }

    fn build_indicator_call(
        &self,
        indicator: &dyn Indicator,
        args: &[Box<AstNode>],
        df: &DataFrame,
    ) -> Result<Expr> {
        // Build args and convert to IndicatorArg based on input types
        let input_types = indicator.input_types();
        let mut indicator_args = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            // Determine if this should be a scalar or series based on input type
            let indicator_arg = if i < input_types.len() {
                match input_types[i] {
                    crate::types::DataType::Integer | crate::types::DataType::Float => {
                        // Check if the AST node is a constant - if so, extract as scalar
                        if let AstNode::Const(value) = arg.as_ref() {
                            match value {
                                Value::Integer(v) => IndicatorArg::Scalar(*v as f64),
                                Value::Float(v) => IndicatorArg::Scalar(*v),
                                _ => {
                                    let arg_expr = self.build(arg, df)?;
                                    IndicatorArg::Series(arg_expr)
                                }
                            }
                        } else {
                            let arg_expr = self.build(arg, df)?;
                            IndicatorArg::Series(arg_expr)
                        }
                    }
                    _ => {
                        let arg_expr = self.build(arg, df)?;
                        IndicatorArg::Series(arg_expr)
                    }
                }
            } else {
                let arg_expr = self.build(arg, df)?;
                IndicatorArg::Series(arg_expr)
            };

            indicator_args.push(indicator_arg);
        }

        // Call try_calculate_vectorized method on Indicator trait
        let result_expr = indicator.try_calculate_vectorized(&indicator_args)
            .ok_or_else(|| TradebiasError::IndicatorError(
                format!("Indicator {} does not implement VectorizedIndicator", indicator.ui_name())
            ))?
            .map_err(|e| TradebiasError::IndicatorError(format!("Indicator calculation failed: {}", e)))?;

        // Return the expression directly instead of evaluating it to a series
        // This avoids stack overflow issues with lit(series) in nested expressions
        Ok(result_expr)
    }

    fn build_primitive_call(
        &self,
        primitive: &dyn Primitive,
        args: &[Box<AstNode>],
        df: &DataFrame,
    ) -> Result<Expr> {
        let arg_exprs: Result<Vec<Expr>> = args.iter().map(|arg| self.build(arg, df)).collect();
        primitive.execute(&arg_exprs?)
            .map_err(|e| TradebiasError::IndicatorError(format!("Primitive execution failed: {}", e)))
    }

    fn create_cache_key(
        &self,
        function: &str,
        args: &[Box<AstNode>],
        _df: &DataFrame,
    ) -> Result<String> {
        Ok(format!("{}-{:?}", function, args))
    }
}
