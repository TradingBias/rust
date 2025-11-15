# 06 - Function Registry & Caching

## Goal
Implement the function registry for indicator discovery and the caching system for performance optimization.

## Prerequisites
- **[05-indicators-tier2.md](./05-indicators-tier2.md)** completed
- All indicators implemented (Tier 1 + Tier 2)

## What You'll Create
1. Function Registry (`src/functions/registry.rs`)
2. Indicator Manifest (`src/functions/manifest.rs`)
3. Indicator Cache (`src/data/cache.rs`)

## Implementation Steps

### Step 1: Implement Indicator Manifest

Update `src/functions/manifest.rs`:

```rust
use crate::types::ScaleType;

pub struct IndicatorManifest {
    pub tier1: Vec<IndicatorMetadata>,
    pub tier2: Vec<IndicatorMetadata>,
    pub tier3: Vec<IndicatorMetadata>,
}

pub struct IndicatorMetadata {
    pub alias: String,
    pub ui_name: String,
    pub scale_type: ScaleType,
    pub value_range: Option<(f64, f64)>,
    pub discovery_weight: f64,  // 0.0-1.0 for GA selection
}

impl IndicatorManifest {
    pub fn default() -> Self {
        Self {
            tier1: Self::build_tier1(),
            tier2: Self::build_tier2(),
            tier3: Vec::new(),
        }
    }

    fn build_tier1() -> Vec<IndicatorMetadata> {
        vec![
            IndicatorMetadata {
                alias: "SMA".to_string(),
                ui_name: "Simple Moving Average".to_string(),
                scale_type: ScaleType::Price,
                value_range: None,
                discovery_weight: 0.8,
            },
            // ... 9 more Tier 1 indicators
        ]
    }

    fn build_tier2() -> Vec<IndicatorMetadata> {
        vec![
            IndicatorMetadata {
                alias: "WilliamsR".to_string(),
                ui_name: "Williams %R".to_string(),
                scale_type: ScaleType::Oscillator0_100,
                value_range: Some((-100.0, 0.0)),
                discovery_weight: 0.5,
            },
            // ... 19 more Tier 2 indicators
        ]
    }

    pub fn get_all_available(&self) -> Vec<&IndicatorMetadata> {
        self.tier1.iter()
            .chain(self.tier2.iter())
            .chain(self.tier3.iter())
            .collect()
    }

    pub fn add_custom(&mut self, metadata: IndicatorMetadata) {
        self.tier3.push(metadata);
    }
}
```

### Step 2: Implement Function Registry

Update `src/functions/registry.rs`:

```rust
use crate::functions::traits::*;
use crate::functions::primitives::*;
use crate::functions::indicators::*;
use crate::functions::manifest::IndicatorManifest;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FunctionRegistry {
    primitives: HashMap<String, Arc<dyn Primitive>>,
    indicators: HashMap<String, Arc<dyn Indicator>>,
    manifest: IndicatorManifest,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            indicators: HashMap::new(),
            manifest: IndicatorManifest::default(),
        };

        registry.register_primitives();
        registry.register_indicators();
        registry
    }

    fn register_primitives(&mut self) {
        // Register all primitives
        for primitive in get_all_primitives() {
            let alias = primitive.alias().to_string();
            self.primitives.insert(alias, Arc::from(primitive));
        }
    }

    fn register_indicators(&mut self) {
        // Tier 1
        self.register_indicator(Arc::new(SMA));
        self.register_indicator(Arc::new(EMA));
        self.register_indicator(Arc::new(RSI));
        self.register_indicator(Arc::new(MACD));
        self.register_indicator(Arc::new(BollingerBands));
        self.register_indicator(Arc::new(ATR));
        self.register_indicator(Arc::new(Stochastic));
        self.register_indicator(Arc::new(ADX));
        self.register_indicator(Arc::new(OBV));
        self.register_indicator(Arc::new(CCI));

        // Tier 2
        self.register_indicator(Arc::new(WilliamsR));
        // ... register remaining 19 Tier 2 indicators
    }

    fn register_indicator(&mut self, indicator: Arc<dyn Indicator>) {
        let alias = indicator.alias().to_string();
        self.indicators.insert(alias, indicator);
    }

    pub fn get_indicator(&self, alias: &str) -> Option<Arc<dyn Indicator>> {
        self.indicators.get(alias).cloned()
    }

    pub fn get_primitive(&self, alias: &str) -> Option<Arc<dyn Primitive>> {
        self.primitives.get(alias).cloned()
    }

    pub fn list_indicators(&self) -> Vec<String> {
        self.indicators.keys().cloned().collect()
    }

    pub fn list_primitives(&self) -> Vec<String> {
        self.primitives.keys().cloned().collect()
    }
}
```

### Step 3: Implement Indicator Cache

Update `src/data/cache.rs`:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use polars::prelude::*;
use blake3::hash;

pub struct IndicatorCache {
    cache: Arc<RwLock<HashMap<CacheKey, CachedResult>>>,
    max_size_mb: usize,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    pub indicator: String,
    pub params: Vec<i32>,
    pub data_hash: u64,
}

pub struct CachedResult {
    pub result: Series,
    pub timestamp: Instant,
    pub size_bytes: usize,
}

impl IndicatorCache {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size_mb,
        }
    }

    pub fn get(&self, key: &CacheKey) -> Option<Series> {
        self.cache.read().ok()?
            .get(key)
            .map(|r| r.result.clone())
    }

    pub fn insert(&self, key: CacheKey, result: Series) {
        let size = self.estimate_size(&result);
        let cached = CachedResult {
            result,
            timestamp: Instant::now(),
            size_bytes: size,
        };

        let mut cache = self.cache.write().unwrap();

        // Evict old entries if necessary
        while self.total_size(&cache) + size > self.max_size_mb * 1_024 * 1_024 {
            self.evict_oldest(&mut cache);
        }

        cache.insert(key, cached);
    }

    pub fn hash_data(&self, data: &DataFrame) -> u64 {
        // Hash DataFrame for cache key
        let bytes = format!("{:?}", data); // Simplified
        let hash_result = hash(bytes.as_bytes());
        u64::from_le_bytes(hash_result.as_bytes()[0..8].try_into().unwrap())
    }

    fn estimate_size(&self, series: &Series) -> usize {
        series.len() * 8 // Approximate for f64
    }

    fn total_size(&self, cache: &HashMap<CacheKey, CachedResult>) -> usize {
        cache.values().map(|v| v.size_bytes).sum()
    }

    fn evict_oldest(&self, cache: &mut HashMap<CacheKey, CachedResult>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, v)| v.timestamp)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}
```

## Verification

### Test Registry

```rust
#[test]
fn test_registry() {
    let registry = FunctionRegistry::new();

    // Test primitive retrieval
    assert!(registry.get_primitive("SMA").is_some());
    assert!(registry.get_primitive("EMA").is_some());

    // Test indicator retrieval
    assert!(registry.get_indicator("RSI").is_some());
    assert!(registry.get_indicator("MACD").is_some());

    // Verify counts
    assert_eq!(registry.list_primitives().len(), 15);  // 4 MA + 11 others
    assert_eq!(registry.list_indicators().len(), 30);  // 10 Tier 1 + 20 Tier 2
}
```

### Test Cache

```rust
#[test]
fn test_cache() {
    let cache = IndicatorCache::new(100); // 100 MB

    let key = CacheKey {
        indicator: "RSI".to_string(),
        params: vec![14],
        data_hash: 12345,
    };

    let series = Series::new("test", &[1.0, 2.0, 3.0]);

    // Insert and retrieve
    cache.insert(key.clone(), series.clone());
    let retrieved = cache.get(&key);

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().len(), 3);
}
```

## Next Steps

Proceed to **[07-backtesting-engine.md](./07-backtesting-engine.md)** to implement the backtesting engine.
