use crate::types::StrategyResult;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradebiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub in_sample_result: StrategyResult,
    pub out_of_sample_result: StrategyResult,
    pub fold_num: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub method: String,
    pub folds: Vec<ValidationResult>,
    pub aggregate_metrics: HashMap<String, f64>,
}

pub trait ValidationMethod: Send + Sync {
    fn name(&self) -> &str;

    /// Validate strategy using this method
    fn validate(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<AggregatedResult, TradebiasError>;
}
