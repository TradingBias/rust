use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Required OHLCV columns for market data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequiredColumn {
    Open,
    High,
    Low,
    Close,
    Volume,
}

impl RequiredColumn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::High => "high",
            Self::Low => "low",
            Self::Close => "close",
            Self::Volume => "volume",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Open,
            Self::High,
            Self::Low,
            Self::Close,
            Self::Volume,
        ]
    }

    /// Common alternative column names
    pub fn aliases(&self) -> Vec<&'static str> {
        match self {
            Self::Open => vec!["open", "Open", "OPEN", "o"],
            Self::High => vec!["high", "High", "HIGH", "h"],
            Self::Low => vec!["low", "Low", "LOW", "l"],
            Self::Close => vec!["close", "Close", "CLOSE", "c"],
            Self::Volume => vec!["volume", "Volume", "VOLUME", "vol", "Vol", "v"],
        }
    }
}

/// Metadata about loaded CSV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub file_path: String,
    pub num_rows: usize,
    pub num_columns: usize,
    pub columns: Vec<String>,
    pub has_datetime: bool,
    pub datetime_column: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub price_range: (f64, f64),  // (min, max)
    pub volume_range: (f64, f64), // (min, max)
}

/// Data preview for UI display
#[derive(Debug, Clone)]
pub struct DataPreview {
    pub metadata: DatasetMetadata,
    pub first_rows: Vec<Vec<String>>,  // First 10 rows as strings
    pub column_stats: Vec<ColumnStats>,
}

#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub name: String,
    pub dtype: String,
    pub null_count: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
}
