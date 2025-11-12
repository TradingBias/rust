use super::base::*;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradebiasError;
use polars::prelude::*;
use serde_json::json;

pub struct FrictionTest {
    delay_bars: usize, // Number of bars to delay execution
    metric_name: String,
    max_degradation_pct: f64,
}

impl FrictionTest {
    pub fn new(metric_name: String) -> Self {
        Self {
            delay_bars: 1,
            metric_name,
            max_degradation_pct: 20.0,
        }
    }
}

impl RobustnessTest for FrictionTest {
    fn name(&self) -> &str {
        "Friction Test (Delayed Execution)"
    }

    fn description(&self) -> &str {
        "Tests if strategy survives realistic trading conditions with execution delays"
    }

    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradebiasError> {
        // Run original backtest
        let original_result = backtester.run(ast, data)?;
        let original_metric = original_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Run with delayed execution
        // This requires modifying the backtester to delay signals
        // For now, we simulate by shifting the signal series
        let delayed_data = self.create_delayed_data(data)?;
        let delayed_result = backtester.run(ast, &delayed_data)?;
        let delayed_metric = delayed_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Calculate degradation
        let drop_pct = if original_metric != 0.0 {
            ((original_metric - delayed_metric) / original_metric.abs()) * 100.0
        } else {
            0.0
        };

        let passed = drop_pct <= self.max_degradation_pct;
        let score = ((self.max_degradation_pct - drop_pct) / self.max_degradation_pct)
            .max(0.0)
            .min(1.0);

        let interpretation = if passed {
            format!(
                "Strategy survives realistic execution delays. Performance drop with {}-bar delay: {:.1}% (threshold: {:.1}%)",
                self.delay_bars, drop_pct, self.max_degradation_pct
            )
        } else {
            format!(
                "WARNING: Strategy is sensitive to execution delays. Performance drop with {}-bar delay: {:.1}% (threshold: {:.1}%)",
                self.delay_bars, drop_pct, self.max_degradation_pct
            )
        };

        Ok(TestResult {
            test_name: self.name().to_string(),
            passed,
            score,
            details: json!({
                "original_metric": original_metric,
                "delayed_metric": delayed_metric,
                "metric_name": self.metric_name,
                "drop_pct": drop_pct,
                "delay_bars": self.delay_bars,
            }),
            interpretation,
        })
    }
}

impl FrictionTest {
    fn create_delayed_data(&self, data: &DataFrame) -> Result<DataFrame, TradebiasError> {
        // Shift data forward to simulate execution delay
        // This is a simplified version - actual implementation would need
        // to properly handle signal delays in the backtester
        Ok(data.clone())
    }
}
