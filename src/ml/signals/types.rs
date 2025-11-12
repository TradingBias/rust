use polars::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub timestamp: DateTime<Utc>,
    pub bar_index: usize,
    pub direction: SignalDirection,
    pub indicator_values: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignalDirection {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct SignalDataset {
    pub signals: Vec<Signal>,
    pub market_data: DataFrame,
}
