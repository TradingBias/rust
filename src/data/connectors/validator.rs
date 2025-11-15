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
            if !matches!(series.dtype(), DataType::Float64 | DataType::Float32 | DataType::Int64 | DataType::Int32 | DataType::UInt64 | DataType::UInt32) {
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
    fn find_column<'a>(df: &'a DataFrame, required: &RequiredColumn) -> Option<&'a str> {
        let columns = df.get_column_names();
        for alias in required.aliases() {
            if columns.iter().any(|col| col.as_str() == alias) {
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

        let high = df.column(high_col)?.cast(&DataType::Float64)?;
        let low = df.column(low_col)?.cast(&DataType::Float64)?;
        let open = df.column(open_col)?.cast(&DataType::Float64)?;
        let close = df.column(close_col)?.cast(&DataType::Float64)?;

        let high = high.f64()?;
        let low = low.f64()?;
        let open = open.f64()?;
        let close = close.f64()?;

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
