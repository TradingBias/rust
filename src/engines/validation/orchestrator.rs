use super::robustness::{
    base::*,
    monte_carlo::MonteCarloTest,
    parameter_stability::ParameterStabilityTest,
    friction::FrictionTest,
};
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradebiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessReport {
    pub strategy_id: String,
    pub timestamp: String,
    pub test_results: Vec<TestResult>,
    pub overall_score: f64,
    pub passed_all: bool,
    pub summary: String,
}

pub struct ValidationOrchestrator {
    backtester: Backtester,
    tests: Vec<Box<dyn RobustnessTest>>,
}

impl ValidationOrchestrator {
    pub fn new(backtester: Backtester) -> Self {
        let tests: Vec<Box<dyn RobustnessTest>> = vec![
            Box::new(MonteCarloTest::new(1000, "sharpe_ratio".to_string())),
            Box::new(ParameterStabilityTest::new("sharpe_ratio".to_string())),
            Box::new(FrictionTest::new("sharpe_ratio".to_string())),
        ];

        Self { backtester, tests }
    }

    pub fn run_robustness_report(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        strategy_id: String,
    ) -> Result<RobustnessReport, TradebiasError> {
        let mut test_results = Vec::new();

        // Run all tests
        for test in &self.tests {
            println!("Running test: {}", test.name());
            let result = test.run(ast, data, &self.backtester)?;
            test_results.push(result);
        }

        // Calculate overall metrics
        let overall_score = test_results.iter().map(|r| r.score).sum::<f64>() / test_results.len() as f64;
        let passed_all = test_results.iter().all(|r| r.passed);

        let summary = if passed_all {
            format!(
                "Strategy passed all {} robustness tests with overall score {:.1}%",
                test_results.len(),
                overall_score * 100.0
            )
        } else {
            let failed_count = test_results.iter().filter(|r| !r.passed).count();
            format!(
                "Strategy failed {} of {} robustness tests. Overall score: {:.1}%",
                failed_count,
                test_results.len(),
                overall_score * 100.0
            )
        };

        Ok(RobustnessReport {
            strategy_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            test_results,
            overall_score,
            passed_all,
            summary,
        })
    }
}
