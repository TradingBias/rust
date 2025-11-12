use crate::data::types::StrategyResult;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradebiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub score: f64,           // 0.0 to 1.0 (1.0 = perfect)
    pub details: serde_json::Value,
    pub interpretation: String,
}

pub trait RobustnessTest: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    /// Run the robustness test
    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradebiasError>;
}
