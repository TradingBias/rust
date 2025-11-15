use crate::error::{Result, TradebiasError};
use polars::prelude::*;
use std::path::Path;
use super::{
    types::{DatasetMetadata, DataPreview, ColumnStats, RequiredColumn},
    validator::DataValidator,
};
use std::collections::HashMap;

pub struct CsvConnector;

impl CsvConnector {
    /// Load CSV file into DataFrame
    pub fn load<P: AsRef<Path>>(path: P) -> Result<DataFrame> {
        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(path.as_ref().to_path_buf()))?
            .finish()
            .map_err(|e| TradebiasError::DataLoading(format!("Failed to read CSV: {}", e)))?;

        Ok(df)
    }

    /// Load and validate CSV file
    pub fn load_and_validate<P: AsRef<Path>>(
        path: P,
        min_rows: Option<usize>,
    ) -> Result<(DataFrame, HashMap<RequiredColumn, String>)> {
        let df = Self::load(&path)?;

        // Validate OHLCV columns
        let column_map = DataValidator::validate_ohlcv(&df)?;

        // Check minimum rows (default 100 for meaningful backtests)
        let min_rows = min_rows.unwrap_or(100);
        DataValidator::validate_minimum_rows(&df, min_rows)?;

        // Warn about nulls but don't fail
        let null_report = DataValidator::check_nulls(&df)?;
        if !null_report.is_empty() {
            log::warn!("Null values detected: {:?}", null_report);
        }

        Ok((df, column_map))
    }

    /// Create metadata for a loaded DataFrame
    pub fn create_metadata<P: AsRef<Path>>(
        path: P,
        df: &DataFrame,
    ) -> Result<DatasetMetadata> {
        let columns: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();

        // Detect datetime column
        let (has_datetime, datetime_column) = Self::detect_datetime_column(df);

        // Calculate price range (from close)
        let close_col = Self::find_close_column(df);
        let price_range = if let Some(close_name) = close_col {
            let close = df.column(close_name)?.cast(&DataType::Float64)?;
            let close_f64 = close.f64()?;
            let min = close_f64.min().unwrap_or(0.0);
            let max = close_f64.max().unwrap_or(0.0);
            (min, max)
        } else {
            (0.0, 0.0)
        };

        // Calculate volume range
        let volume_col = Self::find_volume_column(df);
        let volume_range = if let Some(vol_name) = volume_col {
            let vol = df.column(vol_name)?.cast(&DataType::Float64)?;
            let vol_f64 = vol.f64()?;
            let min = vol_f64.min().unwrap_or(0.0);
            let max = vol_f64.max().unwrap_or(0.0);
            (min, max)
        } else {
            (0.0, 0.0)
        };

        Ok(DatasetMetadata {
            file_path: path.as_ref().to_string_lossy().to_string(),
            num_rows: df.height(),
            num_columns: df.width(),
            columns,
            has_datetime,
            datetime_column,
            date_range: None, // TODO: Parse datetime column if exists
            price_range,
            volume_range,
        })
    }

    /// Create a preview of the data for UI display
    pub fn create_preview<P: AsRef<Path>>(
        path: P,
        df: &DataFrame,
    ) -> Result<DataPreview> {
        let metadata = Self::create_metadata(&path, df)?;

        // Get first 10 rows as strings
        let num_preview_rows = 10.min(df.height());
        let mut first_rows = Vec::new();

        for i in 0..num_preview_rows {
            let mut row = Vec::new();
            for col_name in df.get_column_names() {
                let series = df.column(col_name)?;
                let value = match series.dtype() {
                    DataType::Float64 | DataType::Float32 => {
                        let s_f64 = series.cast(&DataType::Float64)?;
                        let f64_series = s_f64.f64()?;
                        f64_series.get(i).map(|v| format!("{:.4}", v)).unwrap_or_else(|| "null".to_string())
                    }
                    DataType::Int64 | DataType::Int32 | DataType::UInt64 | DataType::UInt32 => {
                        let s_i64 = series.cast(&DataType::Int64)?;
                        let i64_series = s_i64.i64()?;
                        i64_series.get(i).map(|v| v.to_string()).unwrap_or_else(|| "null".to_string())
                    }
                    DataType::String => {
                        series.str()?.get(i).unwrap_or("null").to_string()
                    }
                    _ => "?".to_string(),
                };
                row.push(value);
            }
            first_rows.push(row);
        }

        // Calculate column stats
        let mut column_stats = Vec::new();
        for col_name in df.get_column_names() {
            let series = df.column(col_name)?;

            // Try to get numeric stats
            let (min, max, mean) = if matches!(series.dtype(),
                DataType::Float64 | DataType::Float32 | DataType::Int64 | DataType::Int32 | DataType::UInt64 | DataType::UInt32
            ) {
                let s_f64 = series.cast(&DataType::Float64).ok();
                if let Some(s) = s_f64 {
                    let f = s.f64().ok();
                    // Extract mean value from Scalar
                    let mean_scalar = s.mean_reduce();
                    let mean_val = mean_scalar.value().extract::<f64>();
                    (
                        f.and_then(|x| x.min()),
                        f.and_then(|x| x.max()),
                        mean_val,
                    )
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

            let stat = ColumnStats {
                name: col_name.to_string(),
                dtype: format!("{:?}", series.dtype()),
                null_count: series.null_count(),
                min,
                max,
                mean,
            };
            column_stats.push(stat);
        }

        Ok(DataPreview {
            metadata,
            first_rows,
            column_stats,
        })
    }

    /// Normalize column names to lowercase standard names
    pub fn normalize_columns(mut df: DataFrame) -> Result<DataFrame> {
        let column_map = DataValidator::validate_ohlcv(&df)?;

        // Rename columns to standard lowercase names
        for (required, actual_name) in column_map {
            let standard_name = required.as_str();
            if actual_name != standard_name {
                df.rename(&actual_name, standard_name.into())
                    .map_err(|e| TradebiasError::DataLoading(format!("Failed to rename column: {}", e)))?;
            }
        }

        Ok(df)
    }

    // Helper functions
    fn detect_datetime_column(df: &DataFrame) -> (bool, Option<String>) {
        let datetime_aliases = ["date", "datetime", "time", "timestamp", "Date", "DateTime"];
        let columns = df.get_column_names();
        for alias in datetime_aliases {
            if columns.iter().any(|col| col.as_str() == alias) {
                return (true, Some(alias.to_string()));
            }
        }
        (false, None)
    }

    fn find_close_column(df: &DataFrame) -> Option<&str> {
        let columns = df.get_column_names();
        RequiredColumn::Close.aliases()
            .iter()
            .find(|&&alias| columns.iter().any(|col| col.as_str() == alias))
            .copied()
    }

    fn find_volume_column(df: &DataFrame) -> Option<&str> {
        let columns = df.get_column_names();
        RequiredColumn::Volume.aliases()
            .iter()
            .find(|&&alias| columns.iter().any(|col| col.as_str() == alias))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn test_create_preview() {
        let df = df! {
            "open" => &[100.0, 101.0, 102.0],
            "high" => &[101.0, 103.0, 104.0],
            "low" => &[99.0, 100.0, 101.0],
            "close" => &[100.5, 102.0, 103.0],
            "volume" => &[1000.0, 1500.0, 1200.0],
        }
        .unwrap();

        let preview = CsvConnector::create_preview("test.csv", &df);
        assert!(preview.is_ok());

        let preview = preview.unwrap();
        assert_eq!(preview.first_rows.len(), 3);
        assert_eq!(preview.metadata.num_rows, 3);
    }

    #[test]
    fn test_normalize_columns() {
        let df = df! {
            "Open" => &[100.0, 101.0],
            "HIGH" => &[101.0, 103.0],
            "low" => &[99.0, 100.0],
            "Close" => &[100.5, 102.0],
            "Vol" => &[1000.0, 1500.0],
        }
        .unwrap();

        let normalized = CsvConnector::normalize_columns(df);
        assert!(normalized.is_ok());

        let df = normalized.unwrap();
        let cols = df.get_column_names();
        assert!(cols.iter().any(|c| c.as_str() == "open"));
        assert!(cols.iter().any(|c| c.as_str() == "high"));
        assert!(cols.iter().any(|c| c.as_str() == "low"));
        assert!(cols.iter().any(|c| c.as_str() == "close"));
        assert!(cols.iter().any(|c| c.as_str() == "volume"));
    }
}
