// src/engines/metrics/profitability.rs
use crate::types::*;
use std::collections::HashMap;

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
