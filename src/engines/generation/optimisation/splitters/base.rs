use super::types::*;
use polars::prelude::*;
use crate::error::TradebiasError;

pub trait DataSplitter: Send + Sync {
    /// Split data into multiple folds
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradeBiasError>;

    /// Get splitter configuration
    fn config(&self) -> &SplitConfig;
}
