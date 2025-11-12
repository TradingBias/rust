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
