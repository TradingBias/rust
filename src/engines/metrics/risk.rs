// src/engines/metrics/risk.rs
use std::collections::HashMap;

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
