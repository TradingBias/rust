use crate::{
    data::IndicatorCache,
    error::{Result, TradebiasError},
    functions::{
        indicators::Indicator,
        primitive::Primitive,
        registry::FunctionRegistry,
    },
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
            Value-Float(f) => lit(*f),
            Value::String(s) => col(s),
            Value::Bool(b) => lit(*b),
        })
    }

    fn build_call(&self, function: &str, args: &[Box<AstNode>], df: &DataFrame) -> Result<Expr> {
        let cache_key = self.create_cache_key(function, args, df)?;
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(col(&cached.name()));
        }

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
        let arg_exprs: Result<Vec<Expr>> = args.iter().map(|arg| self.build(arg, df)).collect();
        let result_expr = indicator.call(df, &arg_exprs?)?;

        let evaluated = df.clone().lazy().with_column(result_expr.alias("result")).collect()?;
        let series = evaluated.column("result")?.clone();

        let cache_key = self.create_cache_key(indicator.name(), args, df)?;
        self.cache.set(cache_key, series.clone());

        Ok(lit(series))
    }

    fn build_primitive_call(
        &self,
        primitive: &dyn Primitive,
        args: &[Box<AstNode>],
        df: &DataFrame,
    ) -> Result<Expr> {
        let arg_exprs: Result<Vec<Expr>> = args.iter().map(|arg| self.build(arg, df)).collect();
        primitive.call(&arg_exprs?)
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
