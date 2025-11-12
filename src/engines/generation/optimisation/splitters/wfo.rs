use super::base::DataSplitter;
use super::types::{DataSplit, SplitConfig, WindowType};
use crate::error::TradebiasError;
use polars::prelude::*;

pub struct WalkForwardSplitter {
    config: SplitConfig,
}

impl WalkForwardSplitter {
    pub fn new(
        in_sample_pct: f64,
        out_of_sample_pct: f64,
        n_folds: usize,
        window_type: WindowType,
    ) -> Self {
        Self {
            config: SplitConfig {
                in_sample_pct,
                out_of_sample_pct,
                n_folds,
                window_type,
            },
        }
    }
}

impl DataSplitter for WalkForwardSplitter {
    fn split(&self, data: &DataFrame) -> Result<Vec<DataSplit>, TradeBiasError> {
        let total_rows = data.height();
        let timestamps = data.column("timestamp")?.datetime()?;

        match self.config.window_type {
            WindowType::Sliding => self.split_sliding(data, total_rows, timestamps),
            WindowType::Anchored => self.split_anchored(data, total_rows, timestamps),
        }
    }

    fn config(&self) -> &SplitConfig {
        &self.config
    }
}

impl WalkForwardSplitter {
    /// Sliding window: Each fold has same IS size
    fn split_sliding(
        &self,
        data: &DataFrame,
        total_rows: usize,
        timestamps: &DatetimeChunked,
    ) -> Result<Vec<DataSplit>, TradeBiasError> {
        let window_size = total_rows / (self.config.n_folds + 1);
        let is_size = (window_size as f64 * self.config.in_sample_pct) as usize;
        let oos_size = window_size - is_size;

        let mut splits = Vec::new();

        for fold in 0..self.config.n_folds {
            let start_idx = fold * window_size;
            let is_end_idx = start_idx + is_size;
            let oos_end_idx = is_end_idx + oos_size;

            if oos_end_idx > total_rows {
                break; // Not enough data for this fold
            }

            let in_sample = data.slice(start_idx as i64, is_size);
            let out_of_sample = data.slice(is_end_idx as i64, oos_size);

            splits.push(DataSplit {
                in_sample,
                out_of_sample,
                fold_num: fold,
                in_sample_start: get_datetime_at_index(timestamps, start_idx)?,
                in_sample_end: get_datetime_at_index(timestamps, is_end_idx - 1)?,
                out_of_sample_start: get_datetime_at_index(timestamps, is_end_idx)?,
                out_of_sample_end: get_datetime_at_index(timestamps, oos_end_idx - 1)?,
            });
        }

        Ok(splits)
    }

    /// Anchored window: IS period grows with each fold
    fn split_anchored(
        &self,
        data: &DataFrame,
        total_rows: usize,
        timestamps: &DatetimeChunked,
    ) -> Result<Vec<DataSplit>, TradeBiasError> {
        let oos_size = total_rows / (self.config.n_folds + 1);

        let mut splits = Vec::new();

        for fold in 0..self.config.n_folds {
            let oos_start_idx = (fold + 1) * oos_size;
            let oos_end_idx = oos_start_idx + oos_size;

            if oos_end_idx > total_rows {
                break;
            }

            // IS grows: from start to OOS start
            let in_sample = data.slice(0, oos_start_idx);
            let out_of_sample = data.slice(oos_start_idx as i64, oos_size);

            splits.push(DataSplit {
                in_sample,
                out_of_sample,
                fold_num: fold,
                in_sample_start: get_datetime_at_index(timestamps, 0)?,
                in_sample_end: get_datetime_at_index(timestamps, oos_start_idx - 1)?,
                out_of_sample_start: get_datetime_at_index(timestamps, oos_start_idx)?,
                out_of_sample_end: get_datetime_at_index(timestamps, oos_end_idx - 1)?,
            });
        }

        Ok(splits)
    }
}
