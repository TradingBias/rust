use crate::{
    data::IndicatorCache,
    error::{Result, TradebiasError},
    engines::evaluation::{ExpressionBuilder, Portfolio},
    functions::registry::FunctionRegistry,
    types::{AstNode, StrategyResult, Value},
    engines::generation::ast::StrategyAST,
};
use polars::prelude::*;
use std::{collections::HashMap, sync::Arc};

pub struct Backtester {
    expression_builder: Arc<ExpressionBuilder>,
    initial_balance: f64,
}

impl Backtester {
    pub fn new(
        registry: Arc<FunctionRegistry>,
        cache: Arc<IndicatorCache>,
        initial_balance: f64,
    ) -> Self {
        Self {
            expression_builder: Arc::new(ExpressionBuilder::new(registry, cache)),
            initial_balance,
        }
    }

    pub fn run(&self, ast: &StrategyAST, data: &DataFrame) -> Result<StrategyResult> {
        let condition = match ast.root.as_ref() {
            AstNode::Rule { condition, .. } => condition,
            _ => return Err(TradebiasError::Validation("StrategyAST root is not a Rule node".to_string())),
        };
        let signal_expr = self.expression_builder.build(condition, data)?;

        let signals = data
            .clone()
            .lazy()
            .with_column(signal_expr.alias("signal"))
            .collect()?;

        let signal_series = signals.column("signal")?;
        let close_series = data.column("close")?;

        let mut portfolio = Portfolio::new(self.initial_balance);

        for i in 0..signal_series.len() {
            let signal = signal_series.f64()?.get(i).unwrap_or(0.0);
            let price = close_series.f64()?.get(i).unwrap_or(0.0);

            portfolio.process_bar(i, signal, price)?;
        }

        let metrics = self.calculate_metrics(&portfolio)?;

        Ok(StrategyResult {
            ast: ast.root.as_ref().clone(),
            metrics,
            trades: portfolio.get_trades().to_vec(),
            equity_curve: portfolio.get_equity_curve().to_vec(),
            in_sample: true,
        })
    }

    fn calculate_metrics(&self, portfolio: &Portfolio) -> Result<HashMap<String, f64>> {
        let mut metrics = HashMap::new();

        let final_balance = portfolio.final_balance();
        let return_pct = (final_balance - self.initial_balance) / self.initial_balance * 100.0;

        metrics.insert("return_pct".to_string(), return_pct);
        metrics.insert("num_trades".to_string(), portfolio.get_trades().len() as f64);
        metrics.insert("final_balance".to_string(), final_balance);

        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::registry::FunctionRegistry;
    use crate::types::Value;
    use polars::df;

    #[test]
    fn test_backtester() {
        let df = df! {
            "close" => &[100.0, 101.0, 102.0, 101.5, 103.0],
        }
        .unwrap();

        let ast = AstNode::Call {
            function: "GreaterThan".to_string(),
            args: vec![
                Box::new(AstNode::Const(Value::String("close".to_string()))),
                Box::new(AstNode::Const(Value::Float(101.0))),
            ],
        };

        let registry = Arc::new(FunctionRegistry::new());
        let cache = Arc::new(IndicatorCache::new(100));
        let backtester = Backtester::new(registry, cache, 10000.0);

        let result = backtester.run(&ast, &df).unwrap();

        assert!(result.trades.len() > 0);
        assert!(result.metrics.contains_key("return_pct"));
    }
}
