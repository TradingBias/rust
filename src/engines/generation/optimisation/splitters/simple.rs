use super::base::DataSplitter;
use super::types::{DataSplit, SplitConfig};
use crate::error::TradebiasError;
use polars::prelude::*;
use chrono::{DateTime, Utc};

pub struct SimpleSplitter {
    config: SplitConfig,
}

impl SimpleSplitter {
    pub fn new(in_sample_pct: f64) -> Self {
        Self {
            config: SplitConfig {
                in_sample_pct,
                out_of_sample_pct: 1.0 - in_sample_pct,
                n_folds: 1,
            },
        }
    }
}

impl DataSplitter for SimpleSplitter {
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradebiasError> {
        let total_rows = data.height();
        let is_rows = (total_rows as f64 * self.config.in_sample_pct) as usize;

        if is_rows == 0 || is_rows >= total_rows {
            return Err(TradebiasError::Validation(
                "Invalid split: in-sample size is 0 or exceeds data size".to_string(),
            ));
        }

        // Split data
        let in_sample = data.slice(0, is_rows);
        let out_of_sample = data.slice(is_rows as i64, total_rows - is_rows);

        // Extract timestamps
        let timestamps = data.column("timestamp")?.datetime()?;
        let is_start = get_datetime_at_index(timestamps, 0)?;
        let is_end = get_datetime_at_index(timestamps, is_rows - 1)?;
        let oos_start = get_datetime_at_index(timestamps, is_rows)?;
        let oos_end = get_datetime_at_index(timestamps, total_rows - 1)?;

        Ok(vec![DataSplit {
            in_sample,
            out_of_sample,
            fold_num: 0,
            in_sample_start: is_start,
            in_sample_end: is_end,
            out_of_sample_start: oos_start,
            out_of_sample_end: oos_end,
        }])
    }

    fn config(&self) -> &SplitConfig {
        &self.config
    }
}

pub fn get_datetime_at_index(series: &DatetimeChunked, idx: usize) -> Result<DateTime<Utc>, TradebiasError> {
    let timestamp_ms = series.get(idx).ok_or_else(|| {
        TradebiasError::Validation(format!("Cannot get timestamp at index {}", idx))
    })?;

    let timestamp_s = timestamp_ms / 1000;
    let datetime = DateTime::<Utc>::from_timestamp(timestamp_s, 0).ok_or_else(|| {
        TradebiasError::Validation(format!("Invalid timestamp: {}", timestamp_ms))
    })?;

    Ok(datetime)
}
