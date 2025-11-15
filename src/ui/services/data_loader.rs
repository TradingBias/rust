use crate::data::{CsvConnector, DataPreview};
use polars::prelude::*;
use std::path::Path;

pub struct DataLoader;

impl DataLoader {
    /// Load CSV file and return DataFrame with preview
    pub fn load_csv(path: &Path) -> Result<(DataFrame, DataPreview), String> {
        // Load and validate CSV
        let (df, _column_map) = CsvConnector::load_and_validate(path, Some(100))
            .map_err(|e| e.to_string())?;

        // Create preview
        let preview = CsvConnector::create_preview(path, &df)
            .map_err(|e| e.to_string())?;

        // Normalize column names to lowercase standard names
        let normalized_df = CsvConnector::normalize_columns(df)
            .map_err(|e| e.to_string())?;

        Ok((normalized_df, preview))
    }

    /// Validate that DataFrame has minimum required rows
    pub fn validate_minimum_rows(df: &DataFrame, min_rows: usize) -> Result<(), String> {
        if df.height() < min_rows {
            return Err(format!(
                "Insufficient data: {} rows, minimum {} required",
                df.height(),
                min_rows
            ));
        }
        Ok(())
    }
}
