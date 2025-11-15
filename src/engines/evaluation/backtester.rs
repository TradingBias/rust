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
        // Build the entire rule (not just the condition)
        // The rule will return numeric signals: 1.0 for long, -1.0 for short, 0.0 for no action
        let signal_expr = self.expression_builder.build(ast.root.as_ref(), data)?;

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
        let trades = portfolio.get_trades();

        // Basic metrics
        metrics.insert("return_pct".to_string(), return_pct);
        metrics.insert("num_trades".to_string(), trades.len() as f64);
        metrics.insert("final_balance".to_string(), final_balance);

        // Drawdown (as percentage)
        metrics.insert("max_drawdown".to_string(), portfolio.max_drawdown * 100.0);

        // Win rate
        if !trades.is_empty() {
            let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count();
            let win_rate = (winning_trades as f64 / trades.len() as f64) * 100.0;
            metrics.insert("win_rate".to_string(), win_rate);
        } else {
            metrics.insert("win_rate".to_string(), 0.0);
        }

        // Profit factor (gross profit / gross loss)
        let gross_profit: f64 = trades.iter().filter(|t| t.profit > 0.0).map(|t| t.profit).sum();
        let gross_loss: f64 = trades.iter().filter(|t| t.profit < 0.0).map(|t| t.profit.abs()).sum();

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
        metrics.insert("profit_factor".to_string(), profit_factor);

        // Sharpe ratio (simplified version using equity curve)
        let sharpe_ratio = self.calculate_sharpe_ratio(portfolio);
        metrics.insert("sharpe_ratio".to_string(), sharpe_ratio);

        Ok(metrics)
    }

    /// Calculate Sharpe ratio from equity curve
    /// Assumes daily returns, annualization factor = sqrt(252)
    fn calculate_sharpe_ratio(&self, portfolio: &Portfolio) -> f64 {
        let equity_curve = portfolio.get_equity_curve();

        if equity_curve.len() < 2 {
            return 0.0;
        }

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..equity_curve.len() {
            let ret = (equity_curve[i] - equity_curve[i - 1]) / equity_curve[i - 1];
            returns.push(ret);
        }

        if returns.is_empty() {
            return 0.0;
        }

        // Calculate mean and std dev
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;

        let std_dev = variance.sqrt();

        if std_dev < 1e-10 {
            return 0.0;
        }

        // Annualized Sharpe ratio (assuming daily data, 252 trading days)
        let sharpe = (mean_return / std_dev) * (252.0_f64).sqrt();

        sharpe
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

        // Use a simple boolean condition (always true) with a long signal action
        // Rule structure: IF condition THEN action ELSE 0.0
        let condition = AstNode::Const(Value::Bool(true)); // Always true
        let action = AstNode::Const(Value::Float(1.0)); // Signal: 1.0 = long

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
        // Note: With constant signal (always true -> 1.0 long), a position opens but never closes,
        // so trades will be empty. This is expected behavior.
        assert!(result.metrics.contains_key("return_pct"), "Should have return_pct metric");
        assert!(result.equity_curve.len() > 0, "Should have equity curve");

        // The constant signal should have opened a position but not closed it
        assert_eq!(result.trades.len(), 0, "No completed trades with constant 1.0 signal");
    }
}
