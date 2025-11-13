use crate::{
    data::IndicatorCache,
    error::{Result, TradebiasError},
    engines::evaluation::{ExpressionBuilder, Portfolio},
    functions::registry::FunctionRegistry,
    types::{AstNode, StrategyResult},
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
    use crate::engines::generation::ast::{StrategyAST, StrategyMetadata};
    use polars::df;

    #[test]
    fn test_backtester() {
        // Create a simple test with price data that trends upward
        let df = df! {
            "close" => &[100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 105.0, 106.0],
            "high" => &[101.0, 102.0, 103.0, 102.5, 104.0, 105.0, 106.0, 107.0],
            "low" => &[99.0, 100.0, 101.0, 100.5, 102.0, 103.0, 104.0, 105.0],
        }
        .unwrap();

        // Use a simple constant float as condition (signal strength)
        // The backtester uses the condition value directly as the signal
        let condition = AstNode::Const(Value::Float(1.0)); // Signal: 1.0 = long
        let action = AstNode::Const(Value::Float(1.0)); // Not used by backtester currently

        let ast_node = AstNode::Rule {
            condition: Box::new(condition),
            action: Box::new(action),
        };

        let ast = StrategyAST {
            root: Box::new(ast_node),
            metadata: StrategyMetadata::default(),
        };

        let registry = Arc::new(FunctionRegistry::new());
        let cache = Arc::new(IndicatorCache::new(100));
        let backtester = Backtester::new(registry, cache, 10000.0);

        let result = backtester.run(&ast, &df).unwrap();

        // Verify backtester ran successfully and produced metrics
        // Note: With constant 1.0 signal, a position opens but never closes,
        // so trades will be empty. This is expected behavior.
        assert!(result.metrics.contains_key("return_pct"), "Should have return_pct metric");
        assert!(result.equity_curve.len() > 0, "Should have equity curve");

        // The constant signal should have opened a position but not closed it
        assert_eq!(result.trades.len(), 0, "No completed trades with constant signal");
    }
}
