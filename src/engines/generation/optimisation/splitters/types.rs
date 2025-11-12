use polars::prelude::*;
use chrono::{DateTime, Utc};

/// Single data split (in-sample + out-of-sample)
#[derive(Debug, Clone)]
pub struct DataSplit {
    pub in_sample: DataFrame,
    pub out_of_sample: DataFrame,
    pub fold_num: usize,
    pub in_sample_start: DateTime<Utc>,
    pub in_sample_end: DateTime<Utc>,
    pub out_of_sample_start: DateTime<Utc>,
    pub out_of_sample_end: DateTime<Utc>,
}

/// Configuration for data splitting
#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub in_sample_pct: f64,    // e.g., 0.7 = 70% IS
    pub out_of_sample_pct: f64, // e.g., 0.3 = 30% OOS
    pub n_folds: usize,         // Number of WFO windows
    pub window_type: WindowType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowType {
    Sliding,  // Each window uses fixed-size IS period
    Anchored, // IS period grows with each window
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            in_sample_pct: 0.7,
            out_of_sample_pct: 0.3,
            n_folds: 5,
            window_type: WindowType::Sliding,
        }
    }
}
