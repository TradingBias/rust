# 08 - Metrics Engine

## Goal
Implement comprehensive performance metrics for strategy evaluation.

## Prerequisites
- **[07-backtesting-engine.md](./07-backtesting-engine.md)** completed
- Backtesting engine working

## What You'll Create
1. Profitability Metrics (`src/engines/metrics/profitability.rs`)
2. Risk Metrics (`src/engines/metrics/risk.rs`)
3. Returns Metrics (`src/engines/metrics/returns.rs`)
4. Metrics Engine (`src/engines/metrics/engine.rs`)

## Key Metrics

### Profitability
- Total Return %
- Win Rate %
- Average Win/Loss
- Profit Factor

### Risk
- Maximum Drawdown %
- Sharpe Ratio
- Sortino Ratio
- Volatility

### Returns
- CAGR (Compound Annual Growth Rate)
- Calmar Ratio
- Recovery Factor

## Quick Implementation

### Profitability Metrics

```rust
// src/engines/metrics/profitability.rs
use crate::types::*;

pub struct ProfitabilityMetrics;

impl ProfitabilityMetrics {
    pub fn calculate(trades: &[Trade], initial_balance: f64) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        if trades.is_empty() {
            return metrics;
        }

        let total_profit: f64 = trades.iter().map(|t| t.profit).sum();
        let winning_trades: Vec<&Trade> = trades.iter().filter(|t| t.profit > 0.0).collect();
        let losing_trades: Vec<&Trade> = trades.iter().filter(|t| t.profit <= 0.0).collect();

        // Total return %
        let return_pct = (total_profit / initial_balance) * 100.0;
        metrics.insert("return_pct".to_string(), return_pct);

        // Win rate
        let win_rate = (winning_trades.len() as f64 / trades.len() as f64) * 100.0;
        metrics.insert("win_rate".to_string(), win_rate);

        // Average win/loss
        if !winning_trades.is_empty() {
            let avg_win: f64 = winning_trades.iter().map(|t| t.profit).sum::<f64>()
                / winning_trades.len() as f64;
            metrics.insert("avg_win".to_string(), avg_win);
        }

        if !losing_trades.is_empty() {
            let avg_loss: f64 = losing_trades.iter().map(|t| t.profit.abs()).sum::<f64>()
                / losing_trades.len() as f64;
            metrics.insert("avg_loss".to_string(), avg_loss);
        }

        // Profit factor
        let gross_profit: f64 = winning_trades.iter().map(|t| t.profit).sum();
        let gross_loss: f64 = losing_trades.iter().map(|t| t.profit.abs()).sum();
        if gross_loss > 0.0 {
            let profit_factor = gross_profit / gross_loss;
            metrics.insert("profit_factor".to_string(), profit_factor);
        }

        metrics
    }
}
```

### Risk Metrics

```rust
// src/engines/metrics/risk.rs
use crate::types::*;

pub struct RiskMetrics;

impl RiskMetrics {
    pub fn calculate(equity_curve: &[f64]) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        if equity_curve.len() < 2 {
            return metrics;
        }

        // Maximum drawdown
        let max_dd = Self::max_drawdown(equity_curve);
        metrics.insert("max_drawdown_pct".to_string(), max_dd);

        // Volatility (std dev of returns)
        let returns = Self::calculate_returns(equity_curve);
        let volatility = Self::std_dev(&returns);
        metrics.insert("volatility".to_string(), volatility);

        // Sharpe ratio (assuming risk-free rate = 0)
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
        if volatility > 0.0 {
            let sharpe = avg_return / volatility;
            metrics.insert("sharpe_ratio".to_string(), sharpe);
        }

        // Sortino ratio (downside deviation)
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();
        if !downside_returns.is_empty() {
            let downside_dev = Self::std_dev(&downside_returns);
            if downside_dev > 0.0 {
                let sortino = avg_return / downside_dev;
                metrics.insert("sortino_ratio".to_string(), sortino);
            }
        }

        metrics
    }

    fn max_drawdown(equity: &[f64]) -> f64 {
        let mut max_dd = 0.0;
        let mut peak = equity[0];

        for &value in equity.iter() {
            if value > peak {
                peak = value;
            }
            let dd = ((peak - value) / peak) * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }

    fn calculate_returns(equity: &[f64]) -> Vec<f64> {
        equity.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect()
    }

    fn std_dev(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }
}
```

### Metrics Engine

```rust
// src/engines/metrics/engine.rs
use crate::types::*;
use crate::engines::metrics::{ProfitabilityMetrics, RiskMetrics};
use std::collections::HashMap;

pub struct MetricsEngine {
    initial_balance: f64,
}

impl MetricsEngine {
    pub fn new(initial_balance: f64) -> Self {
        Self { initial_balance }
    }

    pub fn calculate_all(&self, result: &StrategyResult) -> HashMap<String, f64> {
        let mut all_metrics = HashMap::new();

        // Profitability metrics
        let profit_metrics = ProfitabilityMetrics::calculate(
            &result.trades,
            self.initial_balance
        );
        all_metrics.extend(profit_metrics);

        // Risk metrics
        let risk_metrics = RiskMetrics::calculate(&result.equity_curve);
        all_metrics.extend(risk_metrics);

        // Basic metrics
        all_metrics.insert("num_trades".to_string(), result.trades.len() as f64);
        all_metrics.insert("final_balance".to_string(),
            result.equity_curve.last().copied().unwrap_or(self.initial_balance));

        all_metrics
    }
}
```

## Verification

```rust
#[test]
fn test_metrics_engine() {
    let trades = vec![
        Trade {
            entry_bar: 0,
            exit_bar: 5,
            entry_price: 100.0,
            exit_price: 110.0,
            direction: Direction::Long,
            size: 1.0,
            profit: 10.0,
            exit_reason: ExitReason::TakeProfit,
            fees: 0.0,
        },
        Trade {
            entry_bar: 6,
            exit_bar: 10,
            entry_price: 110.0,
            exit_price: 105.0,
            direction: Direction::Short,
            size: 1.0,
            profit: 5.0,
            exit_reason: ExitReason::Signal,
            fees: 0.0,
        },
    ];

    let equity_curve = vec![10000.0, 10010.0, 10015.0];

    let result = StrategyResult {
        ast: AstNode::Const(Value::Bool(true)),
        metrics: HashMap::new(),
        trades,
        equity_curve,
        in_sample: true,
    };

    let engine = MetricsEngine::new(10000.0);
    let metrics = engine.calculate_all(&result);

    assert!(metrics.contains_key("return_pct"));
    assert!(metrics.contains_key("win_rate"));
    assert!(metrics.contains_key("max_drawdown_pct"));
    assert!(metrics.contains_key("sharpe_ratio"));
}
```

## Next Steps

Proceed to **[09-code-generation.md](./09-code-generation.md)** to implement MQL5 code generation.
