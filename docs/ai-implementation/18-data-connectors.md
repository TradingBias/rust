# 18 - Data Connectors & Market Data Management

## Goal
Implement flexible data connectors for loading market data from multiple sources (CSV, MT5, Supabase) with validation, caching, and normalization. This provides a unified interface for market data access.

## Prerequisites
- **01-architecture.md** - Project structure
- **02-type-system.md** - Core types
- **17-configuration-system.md** - Configuration management

## What You'll Create
1. `DataConnector` trait - Unified interface for all data sources
2. `CsvConnector` - Load CSV files with validation
3. `SupabaseConnector` - Cloud storage integration
4. `MT5Connector` - MetaTrader 5 connection (concept)
5. `DataCache` - Intelligent caching system
6. `DataValidator` - OHLCV validation and normalization

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│           DataConnector Trait                         │
│  • load() → DataFrame                                 │
│  • validate()                                         │
│  • get_metadata()                                     │
└────────────────┬─────────────────────────────────────┘
                 │
      ┌──────────┴──────────────┬─────────────────┐
      │                         │                  │
┌─────▼──────┐      ┌──────────▼────┐   ┌────────▼─────┐
│    CSV     │      │   Supabase    │   │     MT5      │
│ Connector  │      │   Connector   │   │  Connector   │
├────────────┤      ├───────────────┤   ├──────────────┤
│ • Local    │      │ • Cloud       │   │ • Live feed  │
│   files    │      │   storage     │   │ • Historical │
│ • Fast     │      │ • Metadata    │   │ • Real-time  │
└────────────┘      └───────────────┘   └──────────────┘
        │                    │                   │
        └────────────────────┴───────────────────┘
                             │
                    ┌────────▼────────┐
                    │  DataValidator  │
                    │  • OHLCV check  │
                    │  • Gaps detect  │
                    │  • Normalize    │
                    └─────────────────┘
```

## Key Concepts

### 1. Data Validation
All market data must be validated for:
- Complete OHLCV columns (open, high, low, close, volume)
- Proper data types
- No missing values
- Logical constraints (high >= low, etc.)
- Sorted by timestamp

### 2. Normalization
Different sources use different formats:
- Column names: "Close" vs "close" vs "c"
- Timestamps: Unix, RFC3339, Excel dates
- Timeframes: "1H" vs "H1" vs "60"

### 3. Metadata
Each dataset should include:
- Symbol name
- Timeframe
- Start/end dates
- Number of bars
- Source information

## Implementation

### Step 1: Data Types

Create `src/data/types.rs`:

```rust
use chrono::{DateTime, Utc};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvRecord {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMetadata {
    pub symbol: String,
    pub timeframe: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub num_bars: usize,
    pub source: String,
    pub asset_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarketDataset {
    pub data: DataFrame,
    pub metadata: DataMetadata,
}
```

### Step 2: Data Connector Trait

Create `src/data/connectors/base.rs`:

```rust
use super::super::types::*;
use crate::error::TradeBiasError;
use async_trait::async_trait;
use polars::prelude::*;

#[async_trait]
pub trait DataConnector: Send + Sync {
    /// Load market data
    async fn load(&self, params: &LoadParams) -> Result<MarketDataset, TradeBiasError>;

    /// Check if data source is available
    async fn is_available(&self) -> bool;

    /// Get connector name
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct LoadParams {
    pub symbol: Option<String>,
    pub timeframe: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
}

impl Default for LoadParams {
    fn default() -> Self {
        Self {
            symbol: None,
            timeframe: None,
            start_date: None,
            end_date: None,
            file_path: None,
        }
    }
}
```

### Step 3: CSV Connector

Create `src/data/connectors/csv.rs`:

```rust
use super::base::*;
use super::super::types::*;
use crate::error::TradeBiasError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use polars::prelude::*;
use std::path::Path;

pub struct CsvConnector {
    base_path: String,
}

impl CsvConnector {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    fn parse_datetime_column(&self, df: &DataFrame) -> Result<Series, TradeBiasError> {
        // Try different column names
        let possible_names = ["timestamp", "time", "datetime", "date"];

        for name in &possible_names {
            if let Ok(col) = df.column(name) {
                return self.convert_to_datetime(col, name);
            }
        }

        Err(TradeBiasError::Validation(
            "No timestamp column found".to_string(),
        ))
    }

    fn convert_to_datetime(&self, series: &Series, col_name: &str) -> Result<Series, TradeBiasError> {
        // Try parsing as string first
        if let Ok(str_series) = series.str() {
            // Try RFC3339 format
            let parsed: Result<Vec<Option<i64>>, _> = str_series
                .into_iter()
                .map(|opt_str| {
                    opt_str
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.timestamp_millis())
                })
                .collect();

            if let Ok(timestamps) = parsed {
                let series = Series::new(col_name, timestamps);
                return Ok(series.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?);
            }
        }

        // Already datetime?
        if matches!(series.dtype(), DataType::Datetime(_, _)) {
            return Ok(series.clone());
        }

        Err(TradeBiasError::Validation(
            "Failed to parse timestamp column".to_string(),
        ))
    }

    fn normalize_column_names(&self, mut df: DataFrame) -> Result<DataFrame, TradeBiasError> {
        let column_mapping = [
            (vec!["Open", "OPEN", "o"], "open"),
            (vec!["High", "HIGH", "h"], "high"),
            (vec!["Low", "LOW", "l"], "low"),
            (vec!["Close", "CLOSE", "c"], "close"),
            (vec!["Volume", "VOLUME", "v", "vol"], "volume"),
        ];

        for (variants, standard) in &column_mapping {
            for variant in variants {
                if df.get_column_names().contains(variant) {
                    df.rename(variant, standard)?;
                    break;
                }
            }
        }

        Ok(df)
    }

    fn validate_ohlcv(&self, df: &DataFrame) -> Result<(), TradeBiasError> {
        // Check required columns
        let required = ["timestamp", "open", "high", "low", "close", "volume"];

        for col_name in &required {
            if df.column(col_name).is_err() {
                return Err(TradeBiasError::Validation(format!(
                    "Missing required column: {}",
                    col_name
                )));
            }
        }

        // Validate OHLC logic
        let high = df.column("high")?.f64()?;
        let low = df.column("low")?.f64()?;
        let open = df.column("open")?.f64()?;
        let close = df.column("close")?.f64()?;

        for i in 0..df.height() {
            let h = high.get(i).ok_or_else(|| {
                TradeBiasError::Validation(format!("Invalid high at row {}", i))
            })?;

            let l = low.get(i).ok_or_else(|| {
                TradeBiasError::Validation(format!("Invalid low at row {}", i))
            })?;

            let o = open.get(i).ok_or_else(|| {
                TradeBiasError::Validation(format!("Invalid open at row {}", i))
            })?;

            let c = close.get(i).ok_or_else(|| {
                TradeBiasError::Validation(format!("Invalid close at row {}", i))
            })?;

            // High should be highest
            if h < l || h < o || h < c {
                return Err(TradeBiasError::Validation(format!(
                    "Invalid OHLC at row {}: high must be >= low, open, close",
                    i
                )));
            }

            // Low should be lowest
            if l > h || l > o || l > c {
                return Err(TradeBiasError::Validation(format!(
                    "Invalid OHLC at row {}: low must be <= high, open, close",
                    i
                )));
            }
        }

        Ok(())
    }

    fn extract_metadata(&self, df: &DataFrame, file_path: &str) -> Result<DataMetadata, TradeBiasError> {
        let timestamps = df.column("timestamp")?.datetime()?;

        let start_ms = timestamps.get(0).ok_or_else(|| {
            TradeBiasError::Validation("Empty dataset".to_string())
        })?;

        let end_ms = timestamps.get(df.height() - 1).ok_or_else(|| {
            TradeBiasError::Validation("Cannot get end timestamp".to_string())
        })?;

        let start_date = DateTime::from_timestamp(start_ms / 1000, 0)
            .ok_or_else(|| TradeBiasError::Validation("Invalid start timestamp".to_string()))?;

        let end_date = DateTime::from_timestamp(end_ms / 1000, 0)
            .ok_or_else(|| TradeBiasError::Validation("Invalid end timestamp".to_string()))?;

        // Extract symbol from filename
        let symbol = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        Ok(DataMetadata {
            symbol,
            timeframe: "unknown".to_string(),
            start_date,
            end_date,
            num_bars: df.height(),
            source: "CSV".to_string(),
            asset_type: None,
        })
    }
}

#[async_trait]
impl DataConnector for CsvConnector {
    async fn load(&self, params: &LoadParams) -> Result<MarketDataset, TradeBiasError> {
        let file_path = params.file_path.as_ref().ok_or_else(|| {
            TradeBiasError::Validation("CSV connector requires file_path".to_string())
        })?;

        // Construct full path
        let full_path = if Path::new(file_path).is_absolute() {
            file_path.clone()
        } else {
            format!("{}/{}", self.base_path, file_path)
        };

        // Read CSV
        let mut df = CsvReader::from_path(&full_path)
            .map_err(|e| TradeBiasError::IO(e.to_string()))?
            .has_header(true)
            .finish()
            .map_err(|e| TradeBiasError::IO(e.to_string()))?;

        // Normalize column names
        df = self.normalize_column_names(df)?;

        // Parse timestamp
        let timestamp_series = self.parse_datetime_column(&df)?;
        df.replace("timestamp", timestamp_series)?;

        // Sort by timestamp
        df = df.sort(["timestamp"], false, false)?;

        // Validate
        self.validate_ohlcv(&df)?;

        // Extract metadata
        let metadata = self.extract_metadata(&df, &full_path)?;

        Ok(MarketDataset { data: df, metadata })
    }

    async fn is_available(&self) -> bool {
        Path::new(&self.base_path).exists()
    }

    fn name(&self) -> &str {
        "CSV"
    }
}
```

### Step 4: Supabase Connector (Conceptual)

Create `src/data/connectors/supabase.rs`:

```rust
use super::base::*;
use super::super::types::*;
use crate::error::TradeBiasError;
use async_trait::async_trait;

pub struct SupabaseConnector {
    url: String,
    api_key: String,
    bucket: String,
}

impl SupabaseConnector {
    pub fn new(url: String, api_key: String, bucket: String) -> Self {
        Self { url, api_key, bucket }
    }

    async fn download_file(&self, file_path: &str) -> Result<Vec<u8>, TradeBiasError> {
        // Construct Supabase Storage URL
        let url = format!(
            "{}/storage/v1/object/public/{}/{}",
            self.url, self.bucket, file_path
        );

        // Download file using reqwest
        let response = reqwest::get(&url)
            .await
            .map_err(|e| TradeBiasError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TradeBiasError::Network(format!(
                "Failed to download file: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TradeBiasError::Network(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}

#[async_trait]
impl DataConnector for SupabaseConnector {
    async fn load(&self, params: &LoadParams) -> Result<MarketDataset, TradeBiasError> {
        let file_path = params.file_path.as_ref().ok_or_else(|| {
            TradeBiasError::Validation("Supabase connector requires file_path".to_string())
        })?;

        // Download file
        let bytes = self.download_file(file_path).await?;

        // Parse JSON or CSV
        let json_str = String::from_utf8(bytes)
            .map_err(|e| TradeBiasError::Validation(format!("Invalid UTF-8: {}", e)))?;

        // Parse as JSON array of OHLCV records
        let records: Vec<OhlcvRecord> = serde_json::from_str(&json_str)
            .map_err(|e| TradeBiasError::Validation(format!("Failed to parse JSON: {}", e)))?;

        // Convert to DataFrame
        let df = self.records_to_dataframe(&records)?;

        // Extract metadata
        let metadata = DataMetadata {
            symbol: params.symbol.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
            timeframe: params.timeframe.clone().unwrap_or_else(|| "unknown".to_string()),
            start_date: records.first().unwrap().timestamp,
            end_date: records.last().unwrap().timestamp,
            num_bars: records.len(),
            source: "Supabase".to_string(),
            asset_type: None,
        };

        Ok(MarketDataset { data: df, metadata })
    }

    async fn is_available(&self) -> bool {
        // Check if we can reach Supabase
        reqwest::get(&self.url).await.is_ok()
    }

    fn name(&self) -> &str {
        "Supabase"
    }
}

impl SupabaseConnector {
    fn records_to_dataframe(&self, records: &[OhlcvRecord]) -> Result<DataFrame, TradeBiasError> {
        let timestamps: Vec<i64> = records.iter().map(|r| r.timestamp.timestamp_millis()).collect();
        let opens: Vec<f64> = records.iter().map(|r| r.open).collect();
        let highs: Vec<f64> = records.iter().map(|r| r.high).collect();
        let lows: Vec<f64> = records.iter().map(|r| r.low).collect();
        let closes: Vec<f64> = records.iter().map(|r| r.close).collect();
        let volumes: Vec<f64> = records.iter().map(|r| r.volume).collect();

        let df = DataFrame::new(vec![
            Series::new("timestamp", timestamps).cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?,
            Series::new("open", opens),
            Series::new("high", highs),
            Series::new("low", lows),
            Series::new("close", closes),
            Series::new("volume", volumes),
        ])
        .map_err(|e| TradeBiasError::Computation(e.to_string()))?;

        Ok(df)
    }
}
```

### Step 5: Data Cache

Create `src/data/cache.rs`:

```rust
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use blake3::Hasher;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    pub source: String,
    pub params_hash: String,
}

pub struct CachedData {
    pub data: DataFrame,
    pub size_bytes: usize,
    pub last_accessed: std::time::Instant,
}

pub struct DataCache {
    cache: Arc<RwLock<HashMap<CacheKey, CachedData>>>,
    max_size_bytes: usize,
}

impl DataCache {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size_bytes: max_size_mb * 1024 * 1024,
        }
    }

    pub fn get(&self, key: &CacheKey) -> Option<DataFrame> {
        let mut cache = self.cache.write().unwrap();

        if let Some(cached) = cache.get_mut(key) {
            cached.last_accessed = std::time::Instant::now();
            Some(cached.data.clone())
        } else {
            None
        }
    }

    pub fn put(&self, key: CacheKey, data: DataFrame) {
        let size_bytes = self.estimate_size(&data);

        let mut cache = self.cache.write().unwrap();

        // Evict if necessary
        while self.current_size(&cache) + size_bytes > self.max_size_bytes {
            self.evict_lru(&mut cache);
        }

        cache.insert(
            key,
            CachedData {
                data,
                size_bytes,
                last_accessed: std::time::Instant::now(),
            },
        );
    }

    fn estimate_size(&self, df: &DataFrame) -> usize {
        // Rough estimate: num_rows * num_cols * 8 bytes
        df.height() * df.width() * 8
    }

    fn current_size(&self, cache: &HashMap<CacheKey, CachedData>) -> usize {
        cache.values().map(|c| c.size_bytes).sum()
    }

    fn evict_lru(&self, cache: &mut HashMap<CacheKey, CachedData>) {
        if let Some((key, _)) = cache
            .iter()
            .min_by_key(|(_, v)| v.last_accessed)
        {
            let key = key.clone();
            cache.remove(&key);
        }
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

pub fn hash_params(params: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(params.as_bytes());
    hasher.finalize().to_hex().to_string()
}
```

## Usage Example

```rust
use tradebias::data::connectors::{CsvConnector, LoadParams, DataConnector};
use tradebias::data::cache::DataCache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create CSV connector
    let connector = CsvConnector::new("./data".to_string());

    // Load data
    let params = LoadParams {
        file_path: Some("EURUSD_H1.csv".to_string()),
        ..Default::default()
    };

    let dataset = connector.load(&params).await?;

    println!("Loaded {} bars", dataset.metadata.num_bars);
    println!("Symbol: {}", dataset.metadata.symbol);
    println!("Range: {} to {}",
        dataset.metadata.start_date,
        dataset.metadata.end_date
    );

    // Use cache for repeated access
    let cache = DataCache::new(100); // 100 MB cache

    let cache_key = CacheKey {
        source: "csv".to_string(),
        params_hash: hash_params(&format!("{:?}", params)),
    };

    cache.put(cache_key.clone(), dataset.data.clone());

    // Later access (fast)
    if let Some(cached_data) = cache.get(&cache_key) {
        println!("Retrieved from cache: {} rows", cached_data.height());
    }

    Ok(())
}
```

## Verification

### Test 1: CSV Loading
```rust
#[tokio::test]
async fn test_csv_load() {
    let connector = CsvConnector::new("./test_data".to_string());

    let params = LoadParams {
        file_path: Some("test.csv".to_string()),
        ..Default::default()
    };

    let result = connector.load(&params).await;
    assert!(result.is_ok());

    let dataset = result.unwrap();
    assert!(dataset.data.height() > 0);
    assert_eq!(dataset.data.width(), 6); // timestamp + OHLCV
}
```

### Test 2: Data Validation
```rust
#[test]
fn test_ohlc_validation() {
    let connector = CsvConnector::new(".".to_string());

    // Create invalid data (high < low)
    let invalid_df = create_invalid_ohlc_data();

    assert!(connector.validate_ohlcv(&invalid_df).is_err());
}
```

## Common Issues

### Issue: Column name mismatch
**Solution**: CSV connector handles this via `normalize_column_names()`. Supports: "Close", "close", "CLOSE", "c".

### Issue: Timestamp parsing fails
**Solution**: Ensure timestamps are in RFC3339 format ("2024-01-01T00:00:00Z") or Unix milliseconds.

### Issue: High/Low validation fails
**Solution**: Check data source quality. Some feeds have errors. May need data cleaning step.

## Next Steps

Proceed to **[19-calibration-signal-engines.md](./19-calibration-signal-engines.md)** for auto-calibration and signal extraction utilities.
