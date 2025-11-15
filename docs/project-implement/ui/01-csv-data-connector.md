# CSV Data Connector Implementation

## Overview

This document details the implementation of the CSV data connector module, which is required for loading and validating market data files for the UI and backtesting engine.

---

## Module Location

**File**: `src/data/connectors/mod.rs` (currently stub)

**New Files**:
- `src/data/connectors/mod.rs` - Module exports
- `src/data/connectors/csv.rs` - CSV loading implementation
- `src/data/connectors/validator.rs` - Data validation
- `src/data/connectors/types.rs` - Connector-specific types

---

## Implementation

### 1. Types (`src/data/connectors/types.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Required OHLCV columns for market data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
```

---

### 2. Validator (`src/data/connectors/validator.rs`)

```rust
use crate::error::{Result, TradebiasError};
use polars::prelude::*;
use super::types::RequiredColumn;
use std::collections::HashMap;

pub struct DataValidator;

impl DataValidator {
    /// Validate that DataFrame has required OHLCV columns
    pub fn validate_ohlcv(df: &DataFrame) -> Result<HashMap<RequiredColumn, String>> {
        let mut column_map = HashMap::new();

        for required in RequiredColumn::all() {
            match Self::find_column(df, &required) {
                Some(col_name) => {
                    column_map.insert(required, col_name.to_string());
                }
                None => {
                    return Err(TradebiasError::DataLoading(format!(
                        "Missing required column: {} (tried aliases: {:?})",
                        required.as_str(),
                        required.aliases()
                    )));
                }
            }
        }

        // Validate column types are numeric
        for (req_col, actual_name) in &column_map {
            let series = df.column(actual_name)?;
            if !matches!(series.dtype(), DataType::Float64 | DataType::Float32 | DataType::Int64 | DataType::Int32) {
                return Err(TradebiasError::DataLoading(format!(
                    "Column '{}' ({}) must be numeric, found {:?}",
                    actual_name,
                    req_col.as_str(),
                    series.dtype()
                )));
            }
        }

        // Validate OHLC relationships
        Self::validate_ohlc_relationships(df, &column_map)?;

        Ok(column_map)
    }

    /// Find column by checking aliases
    fn find_column(df: &DataFrame, required: &RequiredColumn) -> Option<&str> {
        let columns = df.get_column_names();
        for alias in required.aliases() {
            if columns.contains(&alias) {
                return Some(alias);
            }
        }
        None
    }

    /// Validate OHLC relationships (high >= low, high >= open, high >= close, etc.)
    fn validate_ohlc_relationships(
        df: &DataFrame,
        column_map: &HashMap<RequiredColumn, String>,
    ) -> Result<()> {
        let open_col = column_map.get(&RequiredColumn::Open).unwrap();
        let high_col = column_map.get(&RequiredColumn::High).unwrap();
        let low_col = column_map.get(&RequiredColumn::Low).unwrap();
        let close_col = column_map.get(&RequiredColumn::Close).unwrap();

        let high = df.column(high_col)?.f64()?;
        let low = df.column(low_col)?.f64()?;
        let open = df.column(open_col)?.f64()?;
        let close = df.column(close_col)?.f64()?;

        // Check high >= low
        for i in 0..df.height() {
            if let (Some(h), Some(l), Some(o), Some(c)) = (
                high.get(i),
                low.get(i),
                open.get(i),
                close.get(i),
            ) {
                if h < l {
                    return Err(TradebiasError::DataLoading(format!(
                        "Invalid data at row {}: high ({}) < low ({})",
                        i, h, l
                    )));
                }
                if h < o || h < c {
                    return Err(TradebiasError::DataLoading(format!(
                        "Invalid data at row {}: high ({}) < open ({}) or close ({})",
                        i, h, o, c
                    )));
                }
                if l > o || l > c {
                    return Err(TradebiasError::DataLoading(format!(
                        "Invalid data at row {}: low ({}) > open ({}) or close ({})",
                        i, l, o, c
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check for minimum required rows
    pub fn validate_minimum_rows(df: &DataFrame, min_rows: usize) -> Result<()> {
        if df.height() < min_rows {
            return Err(TradebiasError::DataLoading(format!(
                "Insufficient data: {} rows, minimum {} required",
                df.height(),
                min_rows
            )));
        }
        Ok(())
    }

    /// Check for null values in critical columns
    pub fn check_nulls(df: &DataFrame) -> Result<Vec<(String, usize)>> {
        let mut null_report = Vec::new();

        for col_name in df.get_column_names() {
            let series = df.column(col_name)?;
            let null_count = series.null_count();
            if null_count > 0 {
                null_report.push((col_name.to_string(), null_count));
            }
        }

        Ok(null_report)
    }
}
```

---

### 3. CSV Loader (`src/data/connectors/csv.rs`)

```rust
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
            let close = df.column(close_name)?.f64()?;
            let min = close.min().unwrap_or(0.0);
            let max = close.max().unwrap_or(0.0);
            (min, max)
        } else {
            (0.0, 0.0)
        };

        // Calculate volume range
        let volume_col = Self::find_volume_column(df);
        let volume_range = if let Some(vol_name) = volume_col {
            let vol = df.column(vol_name)?.f64()?;
            let min = vol.min().unwrap_or(0.0);
            let max = vol.max().unwrap_or(0.0);
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
                    DataType::Float64 => {
                        series.f64()?.get(i).map(|v| format!("{:.4}", v)).unwrap_or_else(|| "null".to_string())
                    }
                    DataType::Int64 => {
                        series.i64()?.get(i).map(|v| v.to_string()).unwrap_or_else(|| "null".to_string())
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
            let stat = ColumnStats {
                name: col_name.to_string(),
                dtype: format!("{:?}", series.dtype()),
                null_count: series.null_count(),
                min: series.min::<f64>().ok(),
                max: series.max::<f64>().ok(),
                mean: series.mean(),
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
    pub fn normalize_columns(df: DataFrame) -> Result<DataFrame> {
        let column_map = DataValidator::validate_ohlcv(&df)?;

        let mut df = df;

        // Rename columns to standard lowercase names
        for (required, actual_name) in column_map {
            let standard_name = required.as_str();
            if actual_name != standard_name {
                df.rename(actual_name.as_str(), standard_name)
                    .map_err(|e| TradebiasError::DataLoading(format!("Failed to rename column: {}", e)))?;
            }
        }

        Ok(df)
    }

    // Helper functions
    fn detect_datetime_column(df: &DataFrame) -> (bool, Option<String>) {
        let datetime_aliases = ["date", "datetime", "time", "timestamp", "Date", "DateTime"];
        for alias in datetime_aliases {
            if df.get_column_names().contains(&alias) {
                return (true, Some(alias.to_string()));
            }
        }
        (false, None)
    }

    fn find_close_column(df: &DataFrame) -> Option<&str> {
        RequiredColumn::Close.aliases()
            .iter()
            .find(|&&alias| df.get_column_names().contains(&alias))
            .copied()
    }

    fn find_volume_column(df: &DataFrame) -> Option<&str> {
        RequiredColumn::Volume.aliases()
            .iter()
            .find(|&&alias| df.get_column_names().contains(&alias))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn test_validate_good_data() {
        let df = df! {
            "open" => &[100.0, 101.0, 102.0],
            "high" => &[101.0, 103.0, 104.0],
            "low" => &[99.0, 100.0, 101.0],
            "close" => &[100.5, 102.0, 103.0],
            "volume" => &[1000.0, 1500.0, 1200.0],
        }
        .unwrap();

        let result = DataValidator::validate_ohlcv(&df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_column() {
        let df = df! {
            "open" => &[100.0, 101.0],
            "high" => &[101.0, 103.0],
            "low" => &[99.0, 100.0],
            // Missing 'close'
            "volume" => &[1000.0, 1500.0],
        }
        .unwrap();

        let result = DataValidator::validate_ohlcv(&df);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_ohlc() {
        let df = df! {
            "open" => &[100.0, 101.0],
            "high" => &[99.0, 103.0], // High < Open at row 0
            "low" => &[99.0, 100.0],
            "close" => &[100.5, 102.0],
            "volume" => &[1000.0, 1500.0],
        }
        .unwrap();

        let result = DataValidator::validate_ohlcv(&df);
        assert!(result.is_err());
    }

    #[test]
    fn test_column_aliases() {
        let df = df! {
            "Open" => &[100.0, 101.0],  // Capital O
            "HIGH" => &[101.0, 103.0],   // All caps
            "low" => &[99.0, 100.0],
            "Close" => &[100.5, 102.0],  // Capital C
            "Vol" => &[1000.0, 1500.0],  // Alias for volume
        }
        .unwrap();

        let result = DataValidator::validate_ohlcv(&df);
        assert!(result.is_ok());
    }
}
```

---

### 4. Module Export (`src/data/connectors/mod.rs`)

```rust
mod csv;
mod types;
mod validator;

pub use csv::CsvConnector;
pub use types::{
    DataPreview,
    DatasetMetadata,
    ColumnStats,
    RequiredColumn,
};
pub use validator::DataValidator;
```

---

### 5. Update Error Types

Add to `src/error.rs` if not already present:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TradebiasError {
    // ... existing variants ...

    #[error("Data loading error: {0}")]
    DataLoading(String),

    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),
}
```

---

### 6. Update Data Module (`src/data/mod.rs`)

```rust
pub mod cache;
pub mod connectors;  // Add this line

pub use cache::IndicatorCache;
pub use connectors::{CsvConnector, DataPreview, DatasetMetadata};
```

---

## Usage Example

```rust
use tradebias::data::connectors::{CsvConnector, DataValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load CSV file
    let (df, column_map) = CsvConnector::load_and_validate("data/EURUSD_H1.csv", None)?;

    println!("Loaded {} rows", df.height());
    println!("Column mapping: {:?}", column_map);

    // Create preview for UI
    let preview = CsvConnector::create_preview("data/EURUSD_H1.csv", &df)?;
    println!("Preview: {:?}", preview);

    // Normalize column names
    let normalized_df = CsvConnector::normalize_columns(df)?;
    println!("Normalized columns: {:?}", normalized_df.get_column_names());

    Ok(())
}
```

---

## Integration with UI

### In `ui/services/data_loader.rs`:

```rust
use crate::data::connectors::{CsvConnector, DataPreview};
use std::path::Path;

pub struct DataLoader;

impl DataLoader {
    pub fn load_csv(path: &Path) -> Result<(DataFrame, DataPreview), String> {
        let (df, _column_map) = CsvConnector::load_and_validate(path, None)
            .map_err(|e| e.to_string())?;

        let preview = CsvConnector::create_preview(path, &df)
            .map_err(|e| e.to_string())?;

        let normalized = CsvConnector::normalize_columns(df)
            .map_err(|e| e.to_string())?;

        Ok((normalized, preview))
    }
}
```

---

## Testing Strategy

### Unit Tests
- ✅ Validate good OHLCV data
- ✅ Reject missing columns
- ✅ Reject invalid OHLC relationships
- ✅ Accept column name aliases
- Test null value detection
- Test minimum row validation

### Integration Tests
- Load real CSV files from `tests/data/`
- Test various CSV formats (comma, semicolon, tab-delimited)
- Test different date formats
- Test international number formats

### Test Data Files
Create `tests/data/` directory with sample files:
- `valid_ohlcv.csv` - Standard format
- `missing_volume.csv` - Missing required column
- `invalid_ohlc.csv` - High < Low violations
- `uppercase_columns.csv` - Test alias matching

---

## Performance Considerations

1. **Lazy Loading**: For very large files, consider using Polars lazy API
2. **Sampling**: For preview, use `.head(10)` instead of full DataFrame
3. **Caching**: Cache loaded DataFrame in UI state to avoid re-reading
4. **Memory**: Monitor memory usage for multi-GB CSV files

---

## Future Enhancements

1. **Multiple File Formats**: Parquet, Arrow, HDF5
2. **Database Connectors**: PostgreSQL, MongoDB
3. **API Connectors**: Yahoo Finance, Alpha Vantage, Binance
4. **Data Cleaning**: Auto-fix common issues (fill nulls, remove duplicates)
5. **Time Zone Handling**: Detect and normalize timezones
6. **Resampling**: Convert intraday data to daily, etc.

---

## Summary

This CSV connector implementation provides:
- ✅ Robust OHLCV validation with flexible column name matching
- ✅ Clear error messages for debugging
- ✅ Data preview generation for UI display
- ✅ Column normalization for consistent downstream processing
- ✅ Comprehensive testing coverage

This module is a critical dependency for the UI and will be implemented in **Phase 1** of the UI development.
