# TradeBias: Python to Rust Complete Migration Guide

## Document Overview

This document provides complete architectural specifications, implementation requirements, and AI-readable migration instructions for converting TradeBias from Python/Rust hybrid to pure Rust with custom indicator implementations.

**Target Audience**: AI agents (Gemini CLI), human architects, implementation teams

**Project Goals**:
1. Migrate entire Python backend to Rust
2. Implement custom indicator algorithms (no MQL5 built-ins)
3. Ensure mathematical consistency between backtesting and live trading
4. Achieve high-performance vectorized backtesting with indicator caching
5. Generate MQL5 EA code with custom indicator library

---

## Table of Contents

1. [Project Architecture](#1-project-architecture)
2. [Indicator System Design](#2-indicator-system-design)
3. [Type System & Traits](#3-type-system--traits)
4. [Implementation Specifications](#4-implementation-specifications)
5. [Migration Instructions (AI-Readable)](#5-migration-instructions-ai-readable)
6. [Testing & Verification](#6-testing--verification)
7. [Performance Optimization](#7-performance-optimization)

---

# 1. Project Architecture

## 1.1 High-Level System Design

```
┌─────────────────────────────────────────────────────────────┐
│                    egui UI (Frontend)                        │
│              Direct Rust Function Calls                      │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                   Core Rust Library                          │
│  ┌──────────────┬──────────────┬───────────────────────┐   │
│  │   Engines    │  Functions   │   ML Pipeline         │   │
│  │  Generation  │  Indicators  │   Features/Labels     │   │
│  │  Evaluation  │  Primitives  │   Models              │   │
│  │  Metrics     │  Risk        │                       │   │
│  └──────────────┴──────────────┴───────────────────────┘   │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┴──────────────┐
        │                              │
┌───────▼────────┐            ┌────────▼──────────┐
│  Code Generator │            │  Data Connectors  │
│  MQL5 EA + MQH  │            │  CSV / Supabase   │
└────────────────┘            └───────────────────┘
```

## 1.2 Directory Structure

```
tradebias/
├── src/
│   ├── lib.rs                          # Public API
│   ├── error.rs                        # Error types
│   ├── types.rs                        # Core types
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   ├── manager.rs                  # Config singleton
│   │   └── schema.rs                   # Serde schemas
│   │
│   ├── data/
│   │   ├── mod.rs
│   │   ├── types.rs                    # Trade, StrategyResult
│   │   ├── connectors/
│   │   │   ├── mod.rs
│   │   │   ├── csv.rs                  # CSV loader
│   │   │   └── supabase.rs             # Cloud storage
│   │   └── cache.rs                    # Indicator cache
│   │
│   ├── functions/
│   │   ├── mod.rs
│   │   ├── registry.rs                 # Function discovery + manifest
│   │   ├── primitives.rs               # 12 core primitives (1 file)
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   ├── trend.rs                # SMA, EMA, MACD (1 file)
│   │   │   ├── momentum.rs             # RSI, Stochastic (1 file)
│   │   │   ├── volatility.rs           # ATR, Bollinger (1 file)
│   │   │   └── volume.rs               # OBV, MFI (1 file)
│   │   ├── manifest.rs                 # Hybrid manifest (tiered)
│   │   └── traits.rs                   # Indicator/Primitive traits
│   │
│   ├── engines/
│   │   ├── mod.rs
│   │   ├── generation/
│   │   │   ├── mod.rs
│   │   │   ├── ast.rs                  # AST types
│   │   │   ├── genome.rs               # Genome encoding
│   │   │   ├── semantic_mapper.rs      # Type-driven generation
│   │   │   ├── evaluator.rs            # Fitness evaluation
│   │   │   └── operators.rs            # Mutation, crossover
│   │   ├── evaluation/
│   │   │   ├── mod.rs
│   │   │   ├── backtester.rs           # Orchestrator
│   │   │   ├── portfolio.rs            # Trade simulation
│   │   │   └── expression.rs           # AST → Polars
│   │   ├── metrics/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs               # Calculation orchestrator
│   │   │   ├── profitability.rs        # Profit metrics
│   │   │   ├── risk.rs                 # Drawdown, volatility
│   │   │   └── returns.rs              # Sharpe, Sortino
│   │   └── validation/
│   │       ├── mod.rs
│   │       └── semantic.rs             # Type checking
│   │
│   ├── ml/
│   │   ├── mod.rs
│   │   ├── features/
│   │   │   ├── mod.rs
│   │   │   └── engineer.rs             # Feature creation
│   │   ├── labeling/
│   │   │   ├── mod.rs
│   │   │   └── triple_barrier.rs       # Signal labeling
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── base.rs                 # Trait definitions
│   │       └── ensemble.rs             # Model combining
│   │
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── mql5_ea.rs                  # EA generation
│   │   └── mql5_indicators.rs          # MQH library generation
│   │
│   └── utils/
│       ├── mod.rs
│       ├── ast_converter.rs            # AST → string
│       └── metadata.rs                 # Indicator metadata
│
├── tests/
│   ├── integration_tests.rs
│   ├── indicator_verification.rs       # Rust vs MQL5 tests
│   └── fixtures/
│       └── sample_data.csv             # Test data
│
├── ui/                                  # Keep existing egui UI
│   └── (existing Rust frontend)
│
└── Cargo.toml
```

## 1.3 Module Dependency Graph

```
┌────────────────────────────────────────────────────────────┐
│  UI Layer (egui)                                           │
│    ↓ calls public API                                      │
└────────────────────────────────────────────────────────────┘
                        ↓
┌────────────────────────────────────────────────────────────┐
│  Public API (lib.rs)                                       │
│  • run_evolution()                                          │
│  • run_backtest()                                           │
│  • train_ml_model()                                         │
│  • generate_mql5_ea()                                       │
└────────────────────────────────────────────────────────────┘
                        ↓
        ┌───────────────┴───────────────┐
        ↓                               ↓
┌───────────────────┐         ┌────────────────────┐
│ Engines           │         │ Functions          │
│ • generation      │◄────────┤ • registry         │
│ • evaluation      │         │ • manifest         │
│ • metrics         │         │ • primitives       │
└─────────┬─────────┘         │ • indicators/*     │
          │                   └────────────────────┘
          ↓                            ↓
┌─────────────────────┐     ┌──────────────────────┐
│ Data                │     │ Code Generator       │
│ • types             │     │ • mql5_ea            │
│ • connectors        │     │ • mql5_indicators    │
│ • cache             │     └──────────────────────┘
└─────────────────────┘
          ↓
┌─────────────────────┐
│ Core Types          │
│ • error.rs          │
│ • types.rs          │
│ • config/           │
└─────────────────────┘
```

**Key Rules**:
- NO circular dependencies
- Data layer only depends on core types
- Functions only depend on core types
- Engines depend on functions + data
- UI only calls public API

---

# 2. Indicator System Design

## 2.1 Dual Implementation Architecture

### Conceptual Model

```
┌─────────────────────────────────────────────────────────────┐
│                    Indicator Trait                          │
│  • calculate_vectorized() → Polars Expr (backtesting)      │
│  • calculate_stateful() → f64 (live trading)                │
│  • metadata: scale, range, arity                            │
└─────────────────────────────────────────────────────────────┘
                           ↓ implements
        ┌──────────────────┴──────────────────┐
        ↓                                      ↓
┌──────────────────────┐          ┌───────────────────────┐
│ Vectorized Mode      │          │ Stateful Mode         │
│ (Backtesting)        │          │ (Live Trading/MQL5)   │
│                      │          │                       │
│ Input: Series        │          │ Input: f64 bar data   │
│ Process: Polars ops  │          │ Process: Maintain     │
│ Output: Expr/Series  │          │          state buffer │
│ Speed: 100-1000x     │          │ Output: f64           │
│                      │          │ Speed: Bar-by-bar     │
└──────────────────────┘          └───────────────────────┘
         ↓                                    ↓
    Used by Backtester                  Used by Code Gen
    (Rust engine)                       (MQL5 EA + MQH)
```

### Key Design Decisions

1. **Same Algorithm, Different Execution**:
   - Mathematical logic MUST be identical
   - Vectorized uses array operations (Polars)
   - Stateful uses circular buffers + incremental updates

2. **Verification Strategy**:
   - Run same data through both modes
   - Compare outputs bar-by-bar
   - Assert differences < 1e-6 (floating point tolerance)

3. **Code Generation**:
   - Stateful mode logic translates directly to MQL5
   - Generate `TradeBias_Indicators.mqh` from Rust implementations
   - EA calls these custom functions

## 2.2 Primitive Set Definition

### Core Primitives (12 Building Blocks)

```rust
// src/functions/primitives.rs

pub enum PrimitiveType {
    // Moving averages (4 methods in 1 primitive)
    MovingAverage(MAMethod),  // SMA, EMA, LWMA, SMMA
    
    // Statistical operations
    Highest,                   // Max over period
    Lowest,                    // Min over period
    Sum,                       // Summation
    StdDev,                    // Standard deviation
    
    // Rate of change
    Momentum,                  // Price change over period
    
    // Transformations
    Shift,                     // Time-shift series
    Absolute,                  // Abs value
    
    // Arithmetic
    Divide,                    // Safe division (avoid /0)
    Multiply,
    Add,
    Subtract,
}

pub enum MAMethod {
    Simple,      // Arithmetic mean
    Exponential, // EMA with smoothing
    Linear,      // Linear-weighted
    Smoothed,    // SMMA (like EMA but different smoothing)
}
```

**Rationale**: These 12 primitives can build ALL 70+ indicators in your library.

**Example Compositions**:
- RSI = `Normalize(Momentum(close, 1))` + averaging gains/losses
- MACD = `EMA(close, 12) - EMA(close, 26)`
- Bollinger Bands = `SMA(close, 20) ± (2 * StdDev(close, 20))`
- Stochastic = `(close - Lowest(low, 14)) / (Highest(high, 14) - Lowest(low, 14)) * 100`

## 2.3 Hybrid Manifest Configuration

### Tiered Manifest Structure

```rust
// src/functions/manifest.rs

pub struct IndicatorManifest {
    pub tier1: Vec<ComposedIndicator>,  // Must-have (10 indicators)
    pub tier2: Vec<ComposedIndicator>,  // Common (20 indicators)
    pub tier3: Vec<ComposedIndicator>,  // User-customizable
}

pub struct ComposedIndicator {
    pub alias: String,           // "RSI", "MACD"
    pub ui_name: String,         // Display name
    pub scale_type: ScaleType,   // Price, Oscillator, etc.
    pub value_range: Option<(f64, f64)>,
    pub composition: CompositionRecipe,
    pub discovery_weight: f64,   // 0.0-1.0, how often GA uses this
}

pub enum CompositionRecipe {
    Primitives(Vec<PrimitiveType>),  // Built from primitives
    Formula(String),                  // Mathematical description
}
```

### Default Manifest Configuration

**Tier 1: Must-Have (10 indicators)**
```rust
const TIER1_INDICATORS: &[&str] = &[
    "SMA",           // Simple Moving Average
    "EMA",           // Exponential Moving Average
    "RSI",           // Relative Strength Index
    "MACD",          // Moving Average Convergence/Divergence
    "BB",            // Bollinger Bands
    "ATR",           // Average True Range
    "Stochastic",    // Stochastic Oscillator
    "ADX",           // Average Directional Index
    "OBV",           // On-Balance Volume
    "CCI",           // Commodity Channel Index
];
```

**Tier 2: Common (20 indicators)**
```rust
const TIER2_INDICATORS: &[&str] = &[
    "WilliamsR",     // Williams %R
    "MFI",           // Money Flow Index
    "ROC",           // Rate of Change
    "DeMarker",      // DeMarker Indicator
    "StdDev",        // Standard Deviation
    "Envelopes",     // Price Envelopes
    "SAR",           // Parabolic SAR
    "Force",         // Force Index
    "Bears",         // Bears Power
    "Bulls",         // Bulls Power
    "Momentum",      // Momentum Indicator
    "DEMA",          // Double EMA
    "TEMA",          // Triple EMA
    "RVI",           // Relative Vigor Index
    "TriX",          // Triple Exponential Average
    "Volumes",       // Volume indicator
    "Chaikin",       // Chaikin Oscillator
    "BWMFI",         // Market Facilitation Index
    "AC",            // Accelerator Oscillator
    "AO",            // Awesome Oscillator
];
```

**Tier 3: User-Customizable (Runtime)**
- User can add/remove indicators via UI
- GA can discover novel combinations from primitives
- Advanced indicators built on-demand

### Configuration API

```rust
impl IndicatorManifest {
    pub fn default() -> Self {
        Self {
            tier1: Self::build_tier1(),
            tier2: Self::build_tier2(),
            tier3: Vec::new(),
        }
    }
    
    pub fn add_to_tier3(&mut self, indicator: ComposedIndicator) {
        self.tier3.push(indicator);
    }
    
    pub fn remove_from_tier3(&mut self, alias: &str) {
        self.tier3.retain(|ind| ind.alias != alias);
    }
    
    pub fn get_all_available(&self) -> Vec<&ComposedIndicator> {
        self.tier1.iter()
            .chain(self.tier2.iter())
            .chain(self.tier3.iter())
            .collect()
    }
}
```

## 2.4 Indicator Caching System

### Cache Architecture

```rust
// src/data/cache.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct IndicatorCache {
    /// Cache key: (indicator_alias, params_hash, data_hash)
    cache: Arc<RwLock<HashMap<CacheKey, CachedResult>>>,
    max_size_mb: usize,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    indicator: String,       // "RSI", "SMA"
    params: Vec<i32>,        // [14] for RSI(14), [20, 2] for BB(20, 2)
    data_hash: u64,          // Hash of input data
}

pub struct CachedResult {
    result: Series,          // Polars series
    timestamp: Instant,
    size_bytes: usize,
}

impl IndicatorCache {
    pub fn get(&self, key: &CacheKey) -> Option<Series> {
        self.cache.read().ok()?.get(key).map(|r| r.result.clone())
    }
    
    pub fn insert(&self, key: CacheKey, result: Series) {
        let size = self.estimate_size(&result);
        let cached = CachedResult {
            result,
            timestamp: Instant::now(),
            size_bytes: size,
        };
        
        let mut cache = self.cache.write().unwrap();
        
        // Eviction policy: LRU if over max_size_mb
        while self.total_size(&cache) + size > self.max_size_mb * 1_024 * 1_024 {
            self.evict_oldest(&mut cache);
        }
        
        cache.insert(key, cached);
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
}
```

### Cache Usage in Evolution

```rust
// When evaluating population during GA:
pub fn evaluate_strategy(
    &self,
    ast: &AstNode,
    data: &DataFrame,
    cache: &IndicatorCache,
) -> Result<StrategyResult> {
    // Parse AST to extract all indicator calls
    let indicators = self.extract_indicators(ast);
    
    // Pre-calculate all indicators (check cache first)
    for (indicator, params) in indicators {
        let key = CacheKey::new(&indicator, &params, data);
        
        if cache.get(&key).is_none() {
            // Not cached, calculate and store
            let result = self.calculate_indicator(&indicator, &params, data)?;
            cache.insert(key, result);
        }
    }
    
    // Now evaluate strategy using cached indicators
    self.backtest_with_cache(ast, data, cache)
}
```

**Benefits**:
- **Performance**: 10-100x speedup when testing multiple strategies on same data
- **Memory Efficiency**: LRU eviction keeps cache bounded
- **Generation Reuse**: Entire population shares indicator calculations

---

# 3. Type System & Traits

## 3.1 Core Types

```rust
// src/types.rs

use serde::{Deserialize, Serialize};
use polars::prelude::*;

/// Value scale information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    Price,              // Follows price (SMA, BB)
    Oscillator0_100,    // 0-100 bounded (RSI, Stochastic)
    OscillatorCentered, // Zero-centered (MACD, Momentum)
    Volatility,         // Small decimals (ATR, StdDev)
    Volume,             // Large integers (OBV)
    Ratio,              // Ratios (Williams %R)
}

/// Data type for expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    NumericSeries,  // Polars Series<f64>
    BoolSeries,     // Polars Series<bool>
    Integer,        // Scalar i32
    Float,          // Scalar f64
}

/// Abstract Syntax Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    Const(Value),
    Call {
        function: String,
        args: Vec<Box<AstNode>>,
    },
    Rule {
        condition: Box<AstNode>,
        action: Box<AstNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// Trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_bar: usize,
    pub exit_bar: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub direction: Direction,
    pub size: f64,
    pub profit: f64,
    pub exit_reason: ExitReason,
    pub fees: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    Signal,
    EndOfData,
}

/// Complete strategy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub ast: AstNode,
    pub metrics: HashMap<String, f64>,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub in_sample: bool,
}
```

## 3.2 Indicator Trait System

```rust
// src/functions/traits.rs

use polars::prelude::*;
use anyhow::Result;

/// Base trait for all indicators
pub trait Indicator: Send + Sync {
    /// Unique identifier
    fn alias(&self) -> &'static str;
    
    /// Display name
    fn ui_name(&self) -> &'static str;
    
    /// Semantic scale type
    fn scale_type(&self) -> ScaleType;
    
    /// Expected value range
    fn value_range(&self) -> Option<(f64, f64)>;
    
    /// Number of parameters
    fn arity(&self) -> usize;
    
    /// Input data types
    fn input_types(&self) -> Vec<DataType>;
    
    /// Output type
    fn output_type(&self) -> DataType {
        DataType::NumericSeries
    }
    
    /// VECTORIZED: Calculate over entire series (backtesting)
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr>;
    
    /// STATEFUL: Calculate single bar with state (live trading)
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64>;
    
    /// Initialize state for stateful calculation
    fn init_state(&self) -> Box<dyn Any>;
    
    /// Generate MQL5 code for this indicator
    fn generate_mql5(&self, args: &[String]) -> String;
}

/// Flexible argument for indicator calls
#[derive(Debug, Clone)]
pub enum IndicatorArg {
    Series(Expr),   // Polars expression
    Scalar(f64),    // Period, threshold, etc.
}

/// Primitive function trait
pub trait Primitive: Send + Sync {
    fn alias(&self) -> &'static str;
    fn ui_name(&self) -> &'static str;
    fn arity(&self) -> usize;
    fn input_types(&self) -> Vec<DataType>;
    fn output_type(&self) -> DataType;
    
    /// Execute primitive (always vectorized)
    fn execute(&self, args: &[Expr]) -> Result<Expr>;
    
    /// Generate MQL5 code
    fn generate_mql5(&self, args: &[String]) -> String;
}
```

## 3.3 Function Registry

```rust
// src/functions/registry.rs

use std::collections::HashMap;
use std::sync::Arc;

pub struct FunctionRegistry {
    primitives: HashMap<String, Arc<dyn Primitive>>,
    indicators: HashMap<String, Arc<dyn Indicator>>,
    manifest: IndicatorManifest,
    config: RegistryConfig,
}

pub struct RegistryConfig {
    /// Probability of selecting primitive vs composed indicator
    pub primitive_weight: f64,  // 0.0-1.0
    
    /// Enable indicator caching
    pub enable_cache: bool,
    
    /// Cache size in MB
    pub cache_size_mb: usize,
}

impl FunctionRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            indicators: HashMap::new(),
            manifest: IndicatorManifest::default(),
            config,
        };
        
        registry.register_primitives();
        registry.register_indicators();
        registry
    }
    
    fn register_primitives(&mut self) {
        // Register all 12 primitives
        self.primitives.insert("MovingAverage".into(), 
            Arc::new(primitives::MovingAverage::new()));
        self.primitives.insert("Highest".into(), 
            Arc::new(primitives::Highest::new()));
        // ... etc
    }
    
    fn register_indicators(&mut self) {
        // Register Tier 1 indicators
        for indicator_alias in TIER1_INDICATORS {
            let indicator = self.build_indicator(indicator_alias);
            self.indicators.insert(indicator_alias.to_string(), indicator);
        }
        
        // Register Tier 2 indicators
        for indicator_alias in TIER2_INDICATORS {
            let indicator = self.build_indicator(indicator_alias);
            self.indicators.insert(indicator_alias.to_string(), indicator);
        }
    }
    
    pub fn get_indicator(&self, alias: &str) -> Option<Arc<dyn Indicator>> {
        self.indicators.get(alias).cloned()
    }
    
    pub fn get_primitive(&self, alias: &str) -> Option<Arc<dyn Primitive>> {
        self.primitives.get(alias).cloned()
    }
    
    pub fn get_function(&self, alias: &str) -> Option<FunctionRef> {
        if let Some(prim) = self.get_primitive(alias) {
            return Some(FunctionRef::Primitive(prim));
        }
        if let Some(ind) = self.get_indicator(alias) {
            return Some(FunctionRef::Indicator(ind));
        }
        None
    }
}

pub enum FunctionRef {
    Primitive(Arc<dyn Primitive>),
    Indicator(Arc<dyn Indicator>),
}
```

---

# 4. Implementation Specifications

## 4.1 Primitive Implementations

### Example: Moving Average Primitive

```rust
// src/functions/primitives.rs

pub struct MovingAverage {
    method: MAMethod,
}

impl Primitive for MovingAverage {
    fn alias(&self) -> &'static str {
        match self.method {
            MAMethod::Simple => "SMA",
            MAMethod::Exponential => "EMA",
            MAMethod::Linear => "LWMA",
            MAMethod::Smoothed => "SMMA",
        }
    }
    
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        if args.len() != 2 {
            bail!("MovingAverage requires 2 args: series, period");
        }
        
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        match self.method {
            MAMethod::Simple => {
                Ok(series.rolling_mean(RollingOptionsFixedWindow {
                    window_size: period,
                    ..Default::default()
                }))
            }
            MAMethod::Exponential => {
                Ok(series.ewm_mean(EWMOptions {
                    span: period,
                    ..Default::default()
                }))
            }
            MAMethod::Linear => {
                // LWMA: weights = [1, 2, 3, ..., period]
                let weights: Vec<f64> = (1..=period)
                    .map(|i| i as f64)
                    .collect();
                let sum_weights: f64 = weights.iter().sum();
                
                // Rolling window with custom weights
                self.weighted_rolling_mean(series, &weights, sum_weights)
            }
            MAMethod::Smoothed => {
                // SMMA: Similar to EMA but with smoothing factor = period
                // SMMA = (SMMA[i-1] * (period - 1) + close[i]) / period
                self.smoothed_ma(series, period)
            }
        }
    }
    
    fn generate_mql5(&self, args: &[String]) -> String {
        let method = match self.method {
            MAMethod::Simple => "MODE_SMA",
            MAMethod::Exponential => "MODE_EMA",
            MAMethod::Linear => "MODE_LWMA",
            MAMethod::Smoothed => "MODE_SMMA",
        };
        
        format!(
            "TB_MA({}, {}, {})",
            args[0], args[1], method
        )
    }
}

impl MovingAverage {
    fn weighted_rolling_mean(
        &self,
        series: &Expr,
        weights: &[f64],
        sum_weights: f64,
    ) -> Result<Expr> {
        // Implementation using Polars rolling_apply or manual calculation
        // This is pseudocode - actual implementation would use Polars ops
        Ok(series.clone()) // Placeholder
    }
    
    fn smoothed_ma(&self, series: &Expr, period: usize) -> Result<Expr> {
        // SMMA calculation
        // First value is SMA, then: SMMA = (SMMA[i-1] * (period - 1) + close[i]) / period
        Ok(series.clone()) // Placeholder
    }
}
```

### All 12 Primitives (Signatures)

```rust
// src/functions/primitives.rs - Complete module

pub struct MovingAverage { method: MAMethod }
pub struct Highest { period: usize }
pub struct Lowest { period: usize }
pub struct Sum { period: usize }
pub struct StdDev { period: usize }
pub struct Momentum { period: usize }
pub struct Shift { offset: i32 }
pub struct Absolute;
pub struct Divide;
pub struct Multiply;
pub struct Add;
pub struct Subtract;

// Each implements Primitive trait with:
// - execute() for vectorized calculation
// - generate_mql5() for code generation
```

## 4.2 Indicator Implementations

### Example: RSI (Complete Dual Implementation)

```rust
// src/functions/indicators/momentum.rs

use std::collections::VecDeque;

pub struct RSI {
    period: usize,
}

/// State for stateful RSI calculation
pub struct RSIState {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    prev_close: Option<f64>,
}

impl Indicator for RSI {
    fn alias(&self) -> &'static str { "RSI" }
    fn ui_name(&self) -> &'static str { "Relative Strength Index" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 2 }
    
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    
    /// VECTORIZED: Calculate RSI over entire series
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => bail!("RSI: first arg must be series"),
        };
        
        let period = match &args[1] {
            IndicatorArg::Scalar(p) => *p as usize,
            _ => bail!("RSI: second arg must be scalar"),
        };
        
        // Step 1: Calculate price changes
        let delta = series.diff(1, Default::default());
        
        // Step 2: Separate gains and losses
        let gains = delta.clone().clip_min(lit(0.0));
        let losses = delta.clip_max(lit(0.0)).abs();
        
        // Step 3: Calculate average gains and losses using SMMA
        let avg_gains = self.smoothed_ma(&gains, period)?;
        let avg_losses = self.smoothed_ma(&losses, period)?;
        
        // Step 4: Calculate RS and RSI
        let rs = avg_gains.clone() / avg_losses.clone();
        let rsi = lit(100.0) - (lit(100.0) / (lit(1.0) + rs));
        
        Ok(rsi)
    }
    
    /// STATEFUL: Calculate RSI for single bar
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<RSIState>()
            .ok_or_else(|| anyhow!("Invalid state type for RSI"))?;
        
        let close = args[0];
        
        // First bar: just store close
        if state.prev_close.is_none() {
            state.prev_close = Some(close);
            return Ok(50.0); // Default RSI on first bar
        }
        
        let prev_close = state.prev_close.unwrap();
        let change = close - prev_close;
        
        // Separate gain and loss
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };
        
        // Add to buffers
        state.gains.push_back(gain);
        state.losses.push_back(loss);
        
        // Keep only 'period' values
        if state.gains.len() > state.period {
            state.gains.pop_front();
            state.losses.pop_front();
        }
        
        // Calculate averages
        if state.gains.len() < state.period {
            // Not enough data yet, return neutral RSI
            state.prev_close = Some(close);
            return Ok(50.0);
        }
        
        // First average: simple mean
        if state.avg_gain.is_none() {
            let sum_gain: f64 = state.gains.iter().sum();
            let sum_loss: f64 = state.losses.iter().sum();
            state.avg_gain = Some(sum_gain / state.period as f64);
            state.avg_loss = Some(sum_loss / state.period as f64);
        } else {
            // Subsequent: smoothed (Wilder's smoothing)
            let period = state.period as f64;
            state.avg_gain = Some(
                (state.avg_gain.unwrap() * (period - 1.0) + gain) / period
            );
            state.avg_loss = Some(
                (state.avg_loss.unwrap() * (period - 1.0) + loss) / period
            );
        }
        
        // Calculate RSI
        let avg_gain = state.avg_gain.unwrap();
        let avg_loss = state.avg_loss.unwrap();
        
        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        
        state.prev_close = Some(close);
        Ok(rsi)
    }
    
    fn init_state(&self) -> Box<dyn Any> {
        Box::new(RSIState {
            period: self.period,
            gains: VecDeque::new(),
            losses: VecDeque::new(),
            avg_gain: None,
            avg_loss: None,
            prev_close: None,
        })
    }
    
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("TB_RSI({}, {})", args[0], args[1])
    }
}

impl RSI {
    fn smoothed_ma(&self, series: &Expr, period: usize) -> Result<Expr> {
        // Wilder's smoothing: SMMA
        // First value = SMA, then: SMMA[i] = (SMMA[i-1] * (n-1) + value[i]) / n
        // This is a simplified placeholder - real impl uses Polars shift/window ops
        Ok(series.clone())
    }
}
```

### Indicator File Structure

```rust
// src/functions/indicators/trend.rs
pub struct SMA { /* ... */ }
pub struct EMA { /* ... */ }
pub struct MACD { /* ... */ }
pub struct BollingerBands { /* ... */ }
// ... all trend indicators in one file

// src/functions/indicators/momentum.rs
pub struct RSI { /* ... */ }
pub struct Stochastic { /* ... */ }
pub struct CCI { /* ... */ }
pub struct WilliamsR { /* ... */ }
// ... all momentum indicators in one file

// src/functions/indicators/volatility.rs
pub struct ATR { /* ... */ }
pub struct StdDev { /* ... */ }
pub struct ADX { /* ... */ }
// ... all volatility indicators in one file

// src/functions/indicators/volume.rs
pub struct OBV { /* ... */ }
pub struct MFI { /* ... */ }
pub struct Volumes { /* ... */ }
// ... all volume indicators in one file
```

## 4.3 MQL5 Code Generation

### MQL5 Indicator Library Template

```rust
// src/codegen/mql5_indicators.rs

pub struct MQL5IndicatorGenerator {
    registry: Arc<FunctionRegistry>,
}

impl MQL5IndicatorGenerator {
    pub fn generate_mqh_library(&self) -> String {
        let mut code = String::new();
        
        // Header
        code.push_str("//+------------------------------------------------------------------+\n");
        code.push_str("//| TradeBias_Indicators.mqh                                         |\n");
        code.push_str("//| Custom indicator implementations for TradeBias                   |\n");
        code.push_str("//| Generated by TradeBias Rust Engine                               |\n");
        code.push_str("//+------------------------------------------------------------------+\n\n");
        
        // Moving Average
        code.push_str(&self.generate_ma_function());
        
        // RSI
        code.push_str(&self.generate_rsi_function());
        
        // MACD
        code.push_str(&self.generate_macd_function());
        
        // ... generate all other indicators
        
        code
    }
    
    fn generate_rsi_function(&self) -> String {
        r#"
//+------------------------------------------------------------------+
//| Relative Strength Index (RSI)                                    |
//| Algorithm matches TradeBias Rust engine exactly                  |
//+------------------------------------------------------------------+
double TB_RSI(const double &prices[], int period, int shift) {
    if(shift + period >= ArraySize(prices)) return 50.0;
    
    // Calculate average gains and losses using Wilder's smoothing
    double avg_gain = 0.0;
    double avg_loss = 0.0;
    
    // First period: simple average
    for(int i = 1; i <= period; i++) {
        double change = prices[shift + i] - prices[shift + i - 1];
        if(change > 0) avg_gain += change;
        else avg_loss += -change;
    }
    avg_gain /= period;
    avg_loss /= period;
    
    // Calculate RS and RSI
    if(avg_loss == 0.0) return 100.0;
    double rs = avg_gain / avg_loss;
    return 100.0 - (100.0 / (1.0 + rs));
}
"#.to_string()
    }
    
    fn generate_ma_function(&self) -> String {
        r#"
//+------------------------------------------------------------------+
//| Moving Average (Multiple methods)                                |
//+------------------------------------------------------------------+
enum MAMethod {
    MODE_SMA = 0,    // Simple
    MODE_EMA = 1,    // Exponential
    MODE_LWMA = 2,   // Linear Weighted
    MODE_SMMA = 3    // Smoothed
};

double TB_MA(const double &prices[], int period, int method, int shift) {
    if(shift + period >= ArraySize(prices)) return 0.0;
    
    switch(method) {
        case MODE_SMA:
            return TB_SMA(prices, period, shift);
        case MODE_EMA:
            return TB_EMA(prices, period, shift);
        case MODE_LWMA:
            return TB_LWMA(prices, period, shift);
        case MODE_SMMA:
            return TB_SMMA(prices, period, shift);
        default:
            return 0.0;
    }
}

double TB_SMA(const double &prices[], int period, int shift) {
    double sum = 0.0;
    for(int i = 0; i < period; i++) {
        sum += prices[shift + i];
    }
    return sum / period;
}

double TB_EMA(const double &prices[], int period, int shift) {
    double alpha = 2.0 / (period + 1.0);
    double ema = prices[shift + period - 1]; // Start with first price
    
    for(int i = shift + period - 2; i >= shift; i--) {
        ema = alpha * prices[i] + (1.0 - alpha) * ema;
    }
    return ema;
}

double TB_LWMA(const double &prices[], int period, int shift) {
    double sum = 0.0;
    double weight_sum = 0.0;
    
    for(int i = 0; i < period; i++) {
        double weight = (period - i);
        sum += prices[shift + i] * weight;
        weight_sum += weight;
    }
    return sum / weight_sum;
}

double TB_SMMA(const double &prices[], int period, int shift) {
    // Wilder's smoothing
    double smma = TB_SMA(prices, period, shift + period - 1);
    
    for(int i = shift + period - 2; i >= shift; i--) {
        smma = (smma * (period - 1) + prices[i]) / period;
    }
    return smma;
}
"#.to_string()
    }
    
    // ... similar generators for all other indicators
}
```

### EA Generation with Custom Indicators

```rust
// src/codegen/mql5_ea.rs

pub struct MQL5EAGenerator {
    strategy_ast: AstNode,
    registry: Arc<FunctionRegistry>,
}

impl MQL5EAGenerator {
    pub fn generate_ea(&self) -> String {
        let mut code = String::new();
        
        // Header and includes
        code.push_str("//+------------------------------------------------------------------+\n");
        code.push_str("//| TradeBias Strategy EA                                            |\n");
        code.push_str("//+------------------------------------------------------------------+\n");
        code.push_str("#property strict\n\n");
        code.push_str("#include \"TradeBias_Indicators.mqh\"\n\n");
        
        // Input parameters
        code.push_str(&self.generate_inputs());
        
        // Global variables
        code.push_str(&self.generate_globals());
        
        // OnInit
        code.push_str(&self.generate_on_init());
        
        // OnTick
        code.push_str(&self.generate_on_tick());
        
        // Strategy condition function
        code.push_str(&self.generate_condition_function());
        
        // Helper functions
        code.push_str(&self.generate_helpers());
        
        code
    }
    
    fn generate_condition_function(&self) -> String {
        let condition_code = self.ast_to_mql5(&self.strategy_ast);
        
        format!(r#"
//+------------------------------------------------------------------+
//| Strategy Entry Condition                                         |
//+------------------------------------------------------------------+
bool CheckEntryCondition() {{
    {}
}}
"#, condition_code)
    }
    
    fn ast_to_mql5(&self, node: &AstNode) -> String {
        match node {
            AstNode::Call { function, args } => {
                let func_ref = self.registry.get_function(function)
                    .expect("Function not found");
                
                let arg_strs: Vec<String> = args.iter()
                    .map(|arg| self.ast_to_mql5(arg))
                    .collect();
                
                match func_ref {
                    FunctionRef::Indicator(ind) => {
                        ind.generate_mql5(&arg_strs)
                    }
                    FunctionRef::Primitive(prim) => {
                        prim.generate_mql5(&arg_strs)
                    }
                }
            }
            AstNode::Const(val) => {
                match val {
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => if *b { "true" } else { "false" },
                    Value::String(s) => format!("\"{}\"", s),
                }
            }
            AstNode::Rule { condition, action } => {
                format!(
                    "if({}) {{ {} }}",
                    self.ast_to_mql5(condition),
                    self.ast_to_mql5(action)
                )
            }
        }
    }
}
```

## 4.4 Testing & Verification

### Golden File Test Structure

```rust
// tests/indicator_verification.rs

use tradebias::functions::indicators::*;
use polars::prelude::*;
use std::fs;

#[test]
fn test_rsi_matches_reference() {
    // Load sample data
    let data = load_sample_data();
    let close_prices = data.column("close").unwrap();
    
    // Calculate using Rust vectorized
    let rsi_indicator = RSI::new(14);
    let rsi_vectorized = rsi_indicator
        .calculate_vectorized(&[
            IndicatorArg::Series(col("close")),
            IndicatorArg::Scalar(14.0),
        ])
        .unwrap();
    
    let result_vectorized = data
        .lazy()
        .with_column(rsi_vectorized.alias("rsi"))
        .collect()
        .unwrap();
    
    // Calculate using Rust stateful
    let mut state = rsi_indicator.init_state();
    let mut rsi_stateful = Vec::new();
    
    for price in close_prices.f64().unwrap().into_iter() {
        if let Some(p) = price {
            let rsi = rsi_indicator
                .calculate_stateful(&[p], state.as_mut())
                .unwrap();
            rsi_stateful.push(rsi);
        }
    }
    
    // Compare vectorized vs stateful
    let vectorized_values: Vec<f64> = result_vectorized
        .column("rsi")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    
    for (i, (v, s)) in vectorized_values
        .iter()
        .zip(rsi_stateful.iter())
        .enumerate()
    {
        let diff = (v - s).abs();
        assert!(
            diff < 1e-6,
            "RSI mismatch at bar {}: vectorized={}, stateful={}, diff={}",
            i, v, s, diff
        );
    }
    
    // Load reference values (from TA-Lib, MQL5, or manually calculated)
    let reference_values = load_reference_rsi_values();
    
    for (i, (calc, ref_val)) in vectorized_values
        .iter()
        .zip(reference_values.iter())
        .enumerate()
    {
        let diff = (calc - ref_val).abs();
        assert!(
            diff < 1e-4, // Slightly more tolerance for reference comparison
            "RSI mismatch vs reference at bar {}: calculated={}, reference={}, diff={}",
            i, calc, ref_val, diff
        );
    }
}

fn load_sample_data() -> DataFrame {
    let df = CsvReader::from_path("tests/fixtures/sample_data.csv")
        .unwrap()
        .has_header(true)
        .finish()
        .unwrap();
    df
}

fn load_reference_rsi_values() -> Vec<f64> {
    // Load pre-calculated RSI values from golden file
    let content = fs::read_to_string("tests/fixtures/rsi_reference_values.txt")
        .unwrap();
    content
        .lines()
        .map(|line| line.parse::<f64>().unwrap())
        .collect()
}
```

---

# 5. Migration Instructions (AI-Readable)

## 5.1 Overview for AI Agent

**Target**: Gemini CLI or similar autonomous code generation agent

**Objective**: Migrate TradeBias from Python backend to pure Rust, implementing custom indicator algorithms for mathematical consistency between backtesting and live trading.

**Verification**: Use `tests/fixtures/sample_data.csv` (daily EURUSD data) to validate all implementations.

**Phasing**: Implement in discrete, testable phases. Each phase must pass all tests before proceeding to next phase.

## 5.2 Phase 1: Project Setup & Core Types

### Task 1.1: Create Rust Project Structure

**Input**: None
**Output**: Complete directory structure as specified in Section 1.2

**Steps**:
1. Create `tradebias/` root directory
2. Run `cargo init --lib`
3. Create all subdirectories:
   - `src/config/`
   - `src/data/connectors/`
   - `src/functions/indicators/`
   - `src/engines/generation/`, `evaluation/`, `metrics/`, `validation/`
   - `src/ml/features/`, `labeling/`, `models/`
   - `src/codegen/`
   - `src/utils/`
   - `tests/fixtures/`
4. Create all `mod.rs` files in each directory

**Verification**:
```bash
cargo check
```
Should compile without errors (even if modules are empty).

### Task 1.2: Implement Core Types

**Input**: Type specifications from Section 3.1
**Output**: `src/types.rs` with all core types

**Steps**:
1. Copy type definitions from Section 3.1 exactly
2. Add all necessary derive macros
3. Implement `Default` where applicable
4. Add unit tests for serialization/deserialization

**Verification**:
```bash
cargo test types
```
All type tests should pass.

### Task 1.3: Implement Error Types

**Input**: None (design from scratch based on best practices)
**Output**: `src/error.rs` with comprehensive error types

**Template**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradebiasError {
    #[error("Invalid AST: {0}")]
    InvalidAst(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Indicator error: {0}")]
    IndicatorError(String),
    
    #[error("Backtest error: {0}")]
    BacktestError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),
    
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TradebiasError>;
```

**Verification**:
```bash
cargo check
```

### Task 1.4: Implement Indicator Trait System

**Input**: Trait specifications from Section 3.2
**Output**: `src/functions/traits.rs`

**Steps**:
1. Define `Indicator` trait with all methods
2. Define `Primitive` trait
3. Define `IndicatorArg` enum
4. Add documentation for each method

**Verification**:
```bash
cargo check
```

## 5.3 Phase 2: Primitive Implementations

### Task 2.1: Implement MovingAverage Primitive

**Input**: Specification from Section 4.1
**Output**: `src/functions/primitives.rs` (partial)

**Steps**:
1. Implement `MovingAverage` struct with `MAMethod` enum
2. Implement `Primitive` trait
3. Implement `execute()` method for all 4 MA types:
   - Simple (arithmetic mean)
   - Exponential (EMA)
   - Linear Weighted (LWMA)
   - Smoothed (SMMA/Wilder's smoothing)
4. Implement `generate_mql5()` method

**Mathematical Specifications**:

**SMA (Simple Moving Average)**:
```
SMA[i] = (sum of last N prices) / N
```

**EMA (Exponential Moving Average)**:
```
α = 2 / (N + 1)
EMA[i] = α * price[i] + (1 - α) * EMA[i-1]
First EMA = SMA of first N values
```

**LWMA (Linear Weighted Moving Average)**:
```
weights = [1, 2, 3, ..., N]
LWMA[i] = (Σ price[i-j] * weights[j]) / (Σ weights[j])
```

**SMMA (Smoothed Moving Average / Wilder's Smoothing)**:
```
First SMMA = SMA of first N values
SMMA[i] = (SMMA[i-1] * (N - 1) + price[i]) / N
```

**Verification Test**:
```rust
#[test]
fn test_moving_average_all_methods() {
    let data = Series::new("close", vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    let period = 5;
    
    // Test SMA
    let sma = MovingAverage::new(MAMethod::Simple);
    let result = sma.execute(&[col("close"), lit(period)]).unwrap();
    // Verify result matches manual calculation
    
    // Test EMA
    let ema = MovingAverage::new(MAMethod::Exponential);
    let result = ema.execute(&[col("close"), lit(period)]).unwrap();
    // Verify result matches manual calculation
    
    // ... test LWMA and SMMA
}
```

### Task 2.2: Implement Statistical Primitives

**Input**: Specifications from Section 2.2
**Output**: `src/functions/primitives.rs` (continued)

**Primitives to Implement**:

**Highest**:
```rust
pub struct Highest;

impl Primitive for Highest {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        Ok(series.rolling_max(RollingOptionsFixedWindow {
            window_size: period,
            ..Default::default()
        }))
    }
}
```

**Lowest**:
```rust
pub struct Lowest;

impl Primitive for Lowest {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        Ok(series.rolling_min(RollingOptionsFixedWindow {
            window_size: period,
            ..Default::default()
        }))
    }
}
```

**Sum**:
```rust
pub struct Sum;

impl Primitive for Sum {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        Ok(series.rolling_sum(RollingOptionsFixedWindow {
            window_size: period,
            ..Default::default()
        }))
    }
}
```

**StdDev (Standard Deviation)**:
```
σ = sqrt(Σ(x - μ)² / N)
where μ = mean
```

```rust
pub struct StdDev;

impl Primitive for StdDev {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        Ok(series.rolling_std(RollingOptionsFixedWindow {
            window_size: period,
            ..Default::default()
        }))
    }
}
```

**Momentum (Rate of Change)**:
```
Momentum[i] = price[i] - price[i-N]
```

```rust
pub struct Momentum;

impl Primitive for Momentum {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let period = extract_literal_int(&args[1])?;
        
        let shifted = series.shift(lit(period));
        Ok(series.clone() - shifted)
    }
}
```

**Verification Test**:
```rust
#[test]
fn test_all_statistical_primitives() {
    let data = Series::new("close", vec![10.0, 12.0, 11.0, 13.0, 15.0, 14.0, 16.0]);
    
    // Test Highest
    let highest = Highest;
    let result = highest.execute(&[col("close"), lit(3)]).unwrap();
    // Expected: [nan, nan, 12.0, 13.0, 15.0, 15.0, 16.0]
    
    // Test Lowest, Sum, StdDev, Momentum similarly
}
```

### Task 2.3: Implement Arithmetic Primitives

**Input**: Basic arithmetic operations
**Output**: Remaining primitives in `src/functions/primitives.rs`

**Shift (Time-shift operator)**:
```rust
pub struct Shift;

impl Primitive for Shift {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        let series = &args[0];
        let offset = extract_literal_int(&args[1])?;
        
        Ok(series.shift(lit(offset)))
    }
}
```

**Arithmetic operations**:
```rust
pub struct Divide;
impl Primitive for Divide {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        Ok(args[0].clone() / args[1].clone())
    }
}

pub struct Multiply;
impl Primitive for Multiply {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        Ok(args[0].clone() * args[1].clone())
    }
}

pub struct Add;
impl Primitive for Add {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        Ok(args[0].clone() + args[1].clone())
    }
}

pub struct Subtract;
impl Primitive for Subtract {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        Ok(args[0].clone() - args[1].clone())
    }
}

pub struct Absolute;
impl Primitive for Absolute {
    fn execute(&self, args: &[Expr]) -> Result<Expr> {
        Ok(args[0].clone().abs())
    }
}
```

**Verification**:
```bash
cargo test primitives
```
All 12 primitives should have passing tests.

## 5.4 Phase 3: Core Indicators (Tier 1)

### Task 3.1: Implement RSI (Complete Dual Implementation)

**Input**: RSI specification from Section 4.2
**Output**: `src/functions/indicators/momentum.rs` (partial)

**Mathematical Specification**:
```
1. Calculate price changes: delta[i] = close[i] - close[i-1]
2. Separate gains and losses:
   gain[i] = max(delta[i], 0)
   loss[i] = max(-delta[i], 0)
3. Calculate average gain and loss using Wilder's smoothing:
   First avg_gain = SMA(gains, period)
   First avg_loss = SMA(losses, period)
   Subsequent: avg_gain[i] = (avg_gain[i-1] * (period-1) + gain[i]) / period
               avg_loss[i] = (avg_loss[i-1] * (period-1) + loss[i]) / period
4. Calculate RS and RSI:
   RS = avg_gain / avg_loss
   RSI = 100 - (100 / (1 + RS))
```

**Implementation Steps**:
1. Create `RSI` struct
2. Implement `Indicator` trait
3. Implement `calculate_vectorized()` using Polars operations
4. Create `RSIState` struct for stateful mode
5. Implement `calculate_stateful()` with state management
6. Implement `init_state()`
7. Implement `generate_mql5()`

**Verification Test**:
```rust
#[test]
fn test_rsi_dual_implementation() {
    let data = load_sample_data(); // From sample_data.csv
    
    let rsi = RSI::new(14);
    
    // Test vectorized
    let vectorized_result = /* calculate using vectorized mode */;
    
    // Test stateful
    let stateful_result = /* calculate bar-by-bar with state */;
    
    // Compare
    for (i, (v, s)) in vectorized_result.iter().zip(stateful_result.iter()).enumerate() {
        assert!(
            (v - s).abs() < 1e-6,
            "RSI mismatch at bar {}: vec={}, state={}",
            i, v, s
        );
    }
}
```

### Task 3.2: Implement SMA, EMA (Simple Indicators)

**Input**: Moving average specifications
**Output**: `src/functions/indicators/trend.rs` (partial)

**SMA**:
```rust
pub struct SMA {
    period: usize,
}

impl Indicator for SMA {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        // Delegate to MovingAverage primitive
        let ma = MovingAverage::new(MAMethod::Simple);
        ma.execute(/* convert args */)
    }
    
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<SMAState>()?;
        
        state.buffer.push_back(args[0]);
        if state.buffer.len() > self.period {
            state.buffer.pop_front();
        }
        
        if state.buffer.len() < self.period {
            return Ok(0.0); // Not enough data
        }
        
        let sum: f64 = state.buffer.iter().sum();
        Ok(sum / self.period as f64)
    }
}

pub struct SMAState {
    buffer: VecDeque<f64>,
}
```

**EMA**:
```rust
pub struct EMA {
    period: usize,
}

pub struct EMAState {
    period: usize,
    prev_ema: Option<f64>,
    init_buffer: VecDeque<f64>,
}

impl Indicator for EMA {
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<EMAState>()?;
        let price = args[0];
        
        // First N values: accumulate for SMA
        if state.prev_ema.is_none() {
            state.init_buffer.push_back(price);
            
            if state.init_buffer.len() < state.period {
                return Ok(price); // Not initialized yet
            }
            
            // Initialize with SMA
            let sum: f64 = state.init_buffer.iter().sum();
            state.prev_ema = Some(sum / state.period as f64);
        }
        
        // Subsequent values: EMA calculation
        let alpha = 2.0 / (state.period as f64 + 1.0);
        let ema = alpha * price + (1.0 - alpha) * state.prev_ema.unwrap();
        state.prev_ema = Some(ema);
        
        Ok(ema)
    }
}
```

### Task 3.3: Implement MACD

**Input**: MACD specification
**Output**: `src/functions/indicators/trend.rs` (continued)

**Mathematical Specification**:
```
MACD Line = EMA(12) - EMA(26)
Signal Line = EMA(MACD, 9)
Histogram = MACD - Signal
```

**Implementation**:
```rust
pub struct MACD {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}

pub struct MACDState {
    fast_ema: EMAState,
    slow_ema: EMAState,
    signal_ema: EMAState,
}

impl Indicator for MACD {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = extract_series(&args[0])?;
        
        // Fast EMA
        let ema_fast = MovingAverage::new(MAMethod::Exponential)
            .execute(&[series.clone(), lit(self.fast_period)])?;
        
        // Slow EMA
        let ema_slow = MovingAverage::new(MAMethod::Exponential)
            .execute(&[series.clone(), lit(self.slow_period)])?;
        
        // MACD line
        let macd_line = ema_fast - ema_slow;
        
        // Signal line (EMA of MACD line)
        let signal = macd_line.ewm_mean(EWMOptions {
            span: self.signal_period,
            ..Default::default()
        });
        
        // Return MACD line (or struct with line, signal, histogram)
        Ok(macd_line)
    }
    
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<MACDState>()?;
        let price = args[0];
        
        // Calculate fast and slow EMAs
        let fast = EMA::new(self.fast_period).calculate_stateful(&[price], &mut state.fast_ema)?;
        let slow = EMA::new(self.slow_period).calculate_stateful(&[price], &mut state.slow_ema)?;
        
        // MACD line
        let macd = fast - slow;
        
        // Signal line (EMA of MACD)
        let signal = EMA::new(self.signal_period).calculate_stateful(&[macd], &mut state.signal_ema)?;
        
        // Return MACD line (or histogram)
        Ok(macd)
    }
}
```

### Task 3.4: Implement Bollinger Bands

**Mathematical Specification**:
```
Middle Band = SMA(close, period)
Upper Band = Middle + (k * StdDev(close, period))
Lower Band = Middle - (k * StdDev(close, period))
where k = number of standard deviations (typically 2)
```

**Implementation**:
```rust
pub struct BollingerBands {
    period: usize,
    deviation: f64,
}

pub struct BBState {
    sma_state: SMAState,
    buffer: VecDeque<f64>,
}

impl Indicator for BollingerBands {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let series = extract_series(&args[0])?;
        
        // Middle band
        let middle = MovingAverage::new(MAMethod::Simple)
            .execute(&[series.clone(), lit(self.period)])?;
        
        // Standard deviation
        let std = StdDev.execute(&[series.clone(), lit(self.period)])?;
        
        // Upper and lower bands
        let upper = middle.clone() + (std.clone() * lit(self.deviation));
        let lower = middle.clone() - (std * lit(self.deviation));
        
        // Return middle band (or struct with all three)
        Ok(middle)
    }
    
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        let state = state.downcast_mut::<BBState>()?;
        let price = args[0];
        
        // Calculate SMA (middle band)
        let middle = SMA::new(self.period).calculate_stateful(&[price], &mut state.sma_state)?;
        
        // Maintain buffer for StdDev calculation
        state.buffer.push_back(price);
        if state.buffer.len() > self.period {
            state.buffer.pop_front();
        }
        
        if state.buffer.len() < self.period {
            return Ok(middle);
        }
        
        // Calculate standard deviation
        let mean: f64 = state.buffer.iter().sum::<f64>() / self.period as f64;
        let variance: f64 = state.buffer
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / self.period as f64;
        let std = variance.sqrt();
        
        // Return middle band (or upper/lower)
        Ok(middle)
    }
}
```

### Task 3.5: Implement Remaining Tier 1 Indicators

**Indicators to Implement**:
- ATR (Average True Range)
- Stochastic
- ADX (Average Directional Index)
- OBV (On-Balance Volume)
- CCI (Commodity Channel Index)

**Process for Each**:
1. Research mathematical formula
2. Implement `calculate_vectorized()`
3. Implement `calculate_stateful()` with state struct
4. Write verification tests
5. Generate MQL5 code

**Verification**:
```bash
cargo test indicators::tier1
```

## 5.5 Phase 4: Function Registry & Manifest

### Task 4.1: Implement Function Registry

**Input**: Registry specification from Section 3.3
**Output**: `src/functions/registry.rs`

**Steps**:
1. Create `FunctionRegistry` struct
2. Implement primitive registration
3. Implement indicator registration
4. Implement lookup methods
5. Add caching support

**Template**:
```rust
pub struct FunctionRegistry {
    primitives: HashMap<String, Arc<dyn Primitive>>,
    indicators: HashMap<String, Arc<dyn Indicator>>,
    manifest: IndicatorManifest,
    config: RegistryConfig,
}

impl FunctionRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            indicators: HashMap::new(),
            manifest: IndicatorManifest::default(),
            config,
        };
        
        registry.register_all();
        registry
    }
    
    fn register_all(&mut self) {
        self.register_primitives();
        self.register_tier1_indicators();
        self.register_tier2_indicators();
    }
    
    fn register_primitives(&mut self) {
        self.register_primitive("SMA", SMA::new());
        self.register_primitive("EMA", EMA::new());
        // ... all 12 primitives
    }
    
    fn register_tier1_indicators(&mut self) {
        for alias in TIER1_INDICATORS {
            let indicator = self.create_indicator(alias);
            self.register_indicator(alias, indicator);
        }
    }
    
    fn register_indicator(&mut self, alias: &str, indicator: Arc<dyn Indicator>) {
        self.indicators.insert(alias.to_string(), indicator);
    }
}
```

### Task 4.2: Implement Indicator Manifest

**Input**: Manifest specification from Section 2.3
**Output**: `src/functions/manifest.rs`

**Steps**:
1. Define `IndicatorManifest` struct
2. Define tier constants
3. Implement configuration methods
4. Add user customization API

**Verification**:
```rust
#[test]
fn test_manifest_configuration() {
    let mut manifest = IndicatorManifest::default();
    
    assert_eq!(manifest.tier1.len(), 10);
    assert_eq!(manifest.tier2.len(), 20);
    
    // Test adding to tier3
    manifest.add_to_tier3(/* custom indicator */);
    assert_eq!(manifest.tier3.len(), 1);
}
```

### Task 4.3: Implement Indicator Cache

**Input**: Cache specification from Section 2.4
**Output**: `src/data/cache.rs`

**Steps**:
1. Create `IndicatorCache` struct with LRU eviction
2. Implement `get()` and `insert()` methods
3. Add cache key hashing
4. Implement size estimation
5. Add cache statistics

**Verification**:
```rust
#[test]
fn test_cache_lru_eviction() {
    let cache = IndicatorCache::new(1); // 1 MB max
    
    // Insert many indicators until eviction happens
    for i in 0..1000 {
        let key = CacheKey::new("RSI", &[14], /* data hash */);
        let result = Series::new("rsi", vec![50.0; 1000]);
        cache.insert(key, result);
    }
    
    // Verify oldest was evicted
    assert!(cache.total_size() <= 1 * 1024 * 1024);
}
```

## 5.6 Phase 5: Backtesting Engine

### Task 5.1: Implement Expression Builder

**Input**: AST structure, function registry
**Output**: `src/engines/evaluation/expression.rs`

**Functionality**:
- Convert `AstNode` to Polars `Expr`
- Look up functions in registry
- Handle type checking
- Generate Polars expressions recursively

**Template**:
```rust
pub struct ExpressionBuilder {
    registry: Arc<FunctionRegistry>,
}

impl ExpressionBuilder {
    pub fn build(&self, ast: &AstNode) -> Result<Expr> {
        match ast {
            AstNode::Const(val) => Ok(self.const_to_expr(val)),
            
            AstNode::Call { function, args } => {
                let func_ref = self.registry
                    .get_function(function)
                    .ok_or_else(|| TradebiasError::FunctionNotFound(function.clone()))?;
                
                let arg_exprs: Vec<_> = args
                    .iter()
                    .map(|arg| self.build(arg))
                    .collect::<Result<_>>()?;
                
                match func_ref {
                    FunctionRef::Indicator(ind) => {
                        ind.calculate_vectorized(&self.exprs_to_args(&arg_exprs))
                    }
                    FunctionRef::Primitive(prim) => {
                        prim.execute(&arg_exprs)
                    }
                }
            }
            
            AstNode::Rule { condition, action } => {
                // Return condition expression (action is metadata)
                self.build(condition)
            }
        }
    }
}
```

### Task 5.2: Implement Portfolio Simulator

**Input**: Signals, risk parameters
**Output**: `src/engines/evaluation/portfolio.rs`

**Functionality**:
- Vectorized trade simulation
- Position management
- Stop-loss / take-profit execution
- Equity curve generation

**Template**:
```rust
pub struct PortfolioSimulator {
    initial_capital: f64,
    commission: f64,
}

impl PortfolioSimulator {
    pub fn simulate(
        &self,
        signals: &Series,
        data: &DataFrame,
    ) -> Result<(Vec<Trade>, Vec<f64>)> {
        let mut trades = Vec::new();
        let mut equity = self.initial_capital;
        let mut equity_curve = vec![equity];
        
        let signal_values = signals.bool()?;
        
        for (i, signal) in signal_values.into_iter().enumerate() {
            if let Some(true) = signal {
                // Entry signal
                let entry_price = data.column("close")?.f64()?.get(i).unwrap();
                
                // Find exit (simplified - real impl handles SL/TP)
                let exit_bar = self.find_exit(i, data)?;
                let exit_price = data.column("close")?.f64()?.get(exit_bar).unwrap();
                
                // Calculate P&L
                let profit = (exit_price - entry_price) * 1.0; // 1 lot
                equity += profit;
                
                trades.push(Trade {
                    entry_bar: i,
                    exit_bar,
                    entry_price,
                    exit_price,
                    direction: Direction::Long,
                    size: 1.0,
                    profit,
                    exit_reason: ExitReason::Signal,
                    fees: self.commission,
                });
            }
            
            equity_curve.push(equity);
        }
        
        Ok((trades, equity_curve))
    }
}
```

### Task 5.3: Implement Backtester

**Input**: Expression builder, portfolio simulator
**Output**: `src/engines/evaluation/backtester.rs`

**Orchestration**:
```rust
pub struct Backtester {
    expression_builder: ExpressionBuilder,
    portfolio_simulator: PortfolioSimulator,
    cache: Arc<IndicatorCache>,
}

impl Backtester {
    pub fn run(
        &self,
        ast: &AstNode,
        data: &DataFrame,
    ) -> Result<StrategyResult> {
        // Build expression
        let condition_expr = self.expression_builder.build(ast)?;
        
        // Evaluate over data
        let signals = data
            .lazy()
            .with_column(condition_expr.alias("signal"))
            .collect()?
            .column("signal")?
            .clone();
        
        // Simulate portfolio
        let (trades, equity_curve) = self.portfolio_simulator
            .simulate(&signals, data)?;
        
        Ok(StrategyResult {
            ast: ast.clone(),
            metrics: HashMap::new(), // Filled by metrics engine
            trades,
            equity_curve,
            in_sample: true,
        })
    }
}
```

### Task 5.4: Verification

**Test**:
```rust
#[test]
fn test_simple_backtest() {
    let data = load_sample_data();
    let registry = FunctionRegistry::new(Default::default());
    let backtester = Backtester::new(registry);
    
    // Simple strategy: RSI(14) > 70
    let ast = AstNode::Rule {
        condition: Box::new(AstNode::Call {
            function: "GreaterThan".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "RSI".to_string(),
                    args: vec![
                        Box::new(AstNode::Call {
                            function: "Close".to_string(),
                            args: vec![],
                        }),
                        Box::new(AstNode::Const(Value::Integer(14))),
                    ],
                }),
                Box::new(AstNode::Const(Value::Float(70.0))),
            ],
        }),
        action: Box::new(AstNode::Call {
            function: "OpenLong".to_string(),
            args: vec![],
        }),
    };
    
    let result = backtester.run(&ast, &data).unwrap();
    
    assert!(!result.trades.is_empty());
    assert!(!result.equity_curve.is_empty());
}
```

## 5.7 Phase 6: Metrics Engine

### Task 6.1: Implement Core Metrics

**Input**: Trade list, equity curve
**Output**: `src/engines/metrics/` (multiple files)

**Profitability Metrics** (`profitability.rs`):
```rust
pub fn net_profit(trades: &[Trade]) -> f64 {
    trades.iter().map(|t| t.profit).sum()
}

pub fn gross_profit(trades: &[Trade]) -> f64 {
    trades.iter().filter(|t| t.profit > 0.0).map(|t| t.profit).sum()
}

pub fn gross_loss(trades: &[Trade]) -> f64 {
    trades.iter().filter(|t| t.profit < 0.0).map(|t| t.profit.abs()).sum()
}

pub fn profit_factor(gross_profit: f64, gross_loss: f64) -> f64 {
    if gross_loss == 0.0 {
        f64::INFINITY
    } else {
        gross_profit / gross_loss
    }
}
```

**Trade Statistics** (`trades.rs`):
```rust
pub fn total_trades(trades: &[Trade]) -> usize {
    trades.len()
}

pub fn winning_trades(trades: &[Trade]) -> usize {
    trades.iter().filter(|t| t.profit > 0.0).count()
}

pub fn losing_trades(trades: &[Trade]) -> usize {
    trades.iter().filter(|t| t.profit < 0.0).count()
}

pub fn win_rate(winning: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (winning as f64 / total as f64) * 100.0
    }
}
```

**Risk Metrics** (`risk.rs`):
```rust
pub fn max_drawdown(equity_curve: &[f64]) -> f64 {
    let mut max_dd = 0.0;
    let mut peak = equity_curve[0];
    
    for &equity in equity_curve.iter() {
        if equity > peak {
            peak = equity;
        }
        let dd = (peak - equity) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    
    max_dd * 100.0 // Return as percentage
}

pub fn sharpe_ratio(
    returns: &[f64],
    risk_free_rate: f64,
    periods_per_year: f64,
) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let excess_return = mean_return - (risk_free_rate / periods_per_year);
    
    let variance = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();
    
    if std_dev == 0.0 {
        return 0.0;
    }
    
    (excess_return / std_dev) * periods_per_year.sqrt()
}
```

### Task 6.2: Implement Metrics Engine with Dependency Resolution

**Input**: Metric definitions
**Output**: `src/engines/metrics/engine.rs`

**Functionality**:
- Topological sorting of metrics by dependencies
- Graceful degradation (skip failed metrics)
- Caching of intermediate results

**Template**:
```rust
pub struct MetricsEngine {
    calculators: HashMap<String, Box<dyn MetricCalculator>>,
}

impl MetricsEngine {
    pub fn calculate_all(&self, result: &StrategyResult) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        let sorted = self.topological_sort();
        
        for metric_id in sorted {
            if let Some(calc) = self.calculators.get(&metric_id) {
                let context = MetricContext {
                    trades: &result.trades,
                    equity_curve: &result.equity_curve,
                    cache: &metrics,
                    risk_free_rate: 0.02,
                    periods_per_year: 252.0,
                };
                
                match calc.calculate(&context) {
                    Ok(value) => {
                        metrics.insert(metric_id.clone(), value);
                    }
                    Err(e) => {
                        tracing::warn!("Metric {} failed: {}", metric_id, e);
                    }
                }
            }
        }
        
        metrics
    }
    
    fn topological_sort(&self) -> Vec<String> {
        // Kahn's algorithm for topological sorting
        // ... implementation
        vec![]
    }
}
```

## 5.8 Phase 7: Code Generation

### Task 7.1: Generate MQL5 Indicator Library

**Input**: All indicator implementations
**Output**: `TradeBias_Indicators.mqh` file

**Process**:
1. For each indicator in registry
2. Generate MQL5 function signature
3. Convert stateful implementation logic to MQL5
4. Add to MQH file

**Generator**:
```rust
pub struct MQL5IndicatorGenerator {
    registry: Arc<FunctionRegistry>,
}

impl MQL5IndicatorGenerator {
    pub fn generate_library(&self) -> String {
        let mut code = String::new();
        
        code.push_str(&self.header());
        
        // Generate all primitives
        for (alias, primitive) in &self.registry.primitives {
            code.push_str(&self.generate_primitive(primitive));
        }
        
        // Generate all indicators
        for (alias, indicator) in &self.registry.indicators {
            code.push_str(&self.generate_indicator(indicator));
        }
        
        code
    }
    
    fn generate_indicator(&self, indicator: &Arc<dyn Indicator>) -> String {
        // This is a simplified template
        // Real implementation would translate Rust logic to MQL5
        format!(
            r#"
double TB_{}(const double &data[], int period, int shift) {{
    // TODO: Translate stateful implementation to MQL5
    return 0.0;
}}
"#,
            indicator.alias()
        )
    }
}
```

### Task 7.2: Generate Complete EA

**Input**: Strategy AST
**Output**: Complete MQL5 EA file

**Generator**:
```rust
pub struct MQL5EAGenerator {
    ast: AstNode,
    registry: Arc<FunctionRegistry>,
}

impl MQL5EAGenerator {
    pub fn generate(&self) -> String {
        let mut ea = String::new();
        
        ea.push_str(&self.header());
        ea.push_str("#include \"TradeBias_Indicators.mqh\"\n\n");
        ea.push_str(&self.input_parameters());
        ea.push_str(&self.global_variables());
        ea.push_str(&self.on_init());
        ea.push_str(&self.on_tick());
        ea.push_str(&self.condition_function());
        ea.push_str(&self.trade_management());
        
        ea
    }
    
    fn condition_function(&self) -> String {
        let condition = self.ast_to_mql5(&self.ast);
        
        format!(
            r#"
bool CheckEntry() {{
    {}
}}
"#,
            condition
        )
    }
    
    fn ast_to_mql5(&self, node: &AstNode) -> String {
        match node {
            AstNode::Call { function, args } => {
                let func = self.registry.get_function(function).unwrap();
                let arg_strs: Vec<_> = args.iter().map(|a| self.ast_to_mql5(a)).collect();
                
                match func {
                    FunctionRef::Indicator(ind) => ind.generate_mql5(&arg_strs),
                    FunctionRef::Primitive(prim) => prim.generate_mql5(&arg_strs),
                }
            }
            AstNode::Const(val) => self.value_to_mql5(val),
            AstNode::Rule { condition, .. } => self.ast_to_mql5(condition),
        }
    }
}
```

## 5.9 Phase 8: Testing & Verification

### Task 8.1: Golden File Tests

**Create reference data**:
1. Calculate all indicators on `sample_data.csv` using TA-Lib or verified MQL5
2. Save results to `tests/fixtures/golden/`
3. Create tests that compare Rust output to golden files

**Test template**:
```rust
#[test]
fn test_all_indicators_vs_golden_files() {
    let data = load_sample_data();
    
    for indicator_alias in TIER1_INDICATORS {
        let golden_path = format!("tests/fixtures/golden/{}.csv", indicator_alias);
        let golden_values = load_golden_file(&golden_path
        ```rust
#[test]
fn test_all_indicators_vs_golden_files() {
    let data = load_sample_data();
    
    for indicator_alias in TIER1_INDICATORS {
        let golden_path = format!("tests/fixtures/golden/{}.csv", indicator_alias);
        let golden_values = load_golden_file(&golden_path);
        
        let indicator = registry.get_indicator(indicator_alias).unwrap();
        
        // Test vectorized
        let result = indicator.calculate_vectorized(&[
            IndicatorArg::Series(col("close")),
            IndicatorArg::Scalar(14.0),
        ]).unwrap();
        
        let calculated = data
            .lazy()
            .with_column(result.alias("indicator"))
            .collect()
            .unwrap()
            .column("indicator")
            .unwrap()
            .f64()
            .unwrap();
        
        // Compare bar-by-bar
        for (i, (calc, golden)) in calculated
            .into_iter()
            .zip(golden_values.iter())
            .enumerate()
        {
            if let Some(c) = calc {
                let diff = (c - golden).abs();
                assert!(
                    diff < 1e-4,
                    "{} mismatch at bar {}: calculated={}, golden={}, diff={}",
                    indicator_alias, i, c, golden, diff
                );
            }
        }
    }
}
```

### Task 8.2: Vectorized vs Stateful Consistency Tests

**Purpose**: Ensure both calculation modes produce identical results

**Test suite**:
```rust
#[test]
fn test_consistency_all_indicators() {
    let data = load_sample_data();
    let close_prices: Vec<f64> = data
        .column("close")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    
    for indicator_alias in TIER1_INDICATORS.iter().chain(TIER2_INDICATORS.iter()) {
        let indicator = registry.get_indicator(indicator_alias).unwrap();
        
        // Vectorized calculation
        let vectorized = calculate_vectorized(indicator.as_ref(), &data);
        
        // Stateful calculation
        let stateful = calculate_stateful(indicator.as_ref(), &close_prices);
        
        // Compare
        for (i, (v, s)) in vectorized.iter().zip(stateful.iter()).enumerate() {
            let diff = (v - s).abs();
            assert!(
                diff < 1e-6,
                "{} inconsistency at bar {}: vec={}, state={}, diff={}",
                indicator_alias, i, v, s, diff
            );
        }
        
        println!("✓ {} consistency verified", indicator_alias);
    }
}

fn calculate_stateful(indicator: &dyn Indicator, prices: &[f64]) -> Vec<f64> {
    let mut state = indicator.init_state();
    let mut results = Vec::new();
    
    for &price in prices {
        let value = indicator
            .calculate_stateful(&[price], state.as_mut())
            .unwrap();
        results.push(value);
    }
    
    results
}
```

### Task 8.3: MQL5 Consistency Verification

**Process**:
1. Generate MQL5 indicator library
2. Create test EA in MQL5 that outputs indicator values
3. Run on same `sample_data.csv`
4. Compare outputs

**MQL5 Test EA** (manual creation):
```cpp
// TestIndicators.mq5
#property strict
#include "TradeBias_Indicators.mqh"

void OnStart() {
    string filename = "eurusd_daily.csv";
    int handle = FileOpen(filename, FILE_READ|FILE_CSV);
    
    double close_prices[1000];
    int count = 0;
    
    while(!FileIsEnding(handle)) {
        close_prices[count] = FileReadNumber(handle);
        count++;
    }
    FileClose(handle);
    
    // Test RSI
    string output = "rsi_output.csv";
    int out_handle = FileOpen(output, FILE_WRITE|FILE_CSV);
    
    for(int i = 0; i < count; i++) {
        double rsi = TB_RSI(close_prices, 14, i);
        FileWrite(out_handle, rsi);
    }
    FileClose(out_handle);
    
    Print("RSI test complete");
}
```

**Rust Comparison Test**:
```rust
#[test]
#[ignore] // Only run when MQL5 outputs are available
fn test_mql5_consistency() {
    let rust_results = calculate_rsi_rust();
    let mql5_results = load_mql5_output("rsi_output.csv");
    
    for (i, (r, m)) in rust_results.iter().zip(mql5_results.iter()).enumerate() {
        let diff = (r - m).abs();
        assert!(
            diff < 1e-4,
            "RSI Rust vs MQL5 mismatch at bar {}: rust={}, mql5={}, diff={}",
            i, r, m, diff
        );
    }
}
```

### Task 8.4: Integration Tests

**Complete workflow tests**:
```rust
#[test]
fn test_full_strategy_pipeline() {
    // Load data
    let data = CsvConnector::load("tests/fixtures/sample_data.csv").unwrap();
    
    // Create registry
    let registry = FunctionRegistry::new(Default::default());
    
    // Create strategy AST: RSI(14) < 30 AND SMA(20) > SMA(50)
    let ast = create_test_strategy();
    
    // Validate AST
    assert!(ast.type_check(&registry).is_ok());
    
    // Run backtest
    let backtester = Backtester::new(registry.clone());
    let result = backtester.run(&ast, &data).unwrap();
    
    // Verify results
    assert!(!result.trades.is_empty(), "No trades generated");
    assert!(result.equity_curve.len() == data.height());
    
    // Calculate metrics
    let metrics_engine = MetricsEngine::new();
    let metrics = metrics_engine.calculate_all(&result);
    
    assert!(metrics.contains_key("sharpe_ratio"));
    assert!(metrics.contains_key("max_drawdown"));
    assert!(metrics.contains_key("win_rate"));
    
    // Generate MQL5 code
    let ea_gen = MQL5EAGenerator::new(ast, registry);
    let ea_code = ea_gen.generate();
    
    assert!(ea_code.contains("TB_RSI"));
    assert!(ea_code.contains("TB_SMA"));
    assert!(ea_code.contains("#include \"TradeBias_Indicators.mqh\""));
    
    println!("✓ Full pipeline test passed");
}
```

---

# 6. Testing & Verification

## 6.1 Test Data Preparation

### Sample Data Format

**File**: `tests/fixtures/sample_data.csv`

**Format**:
```csv
datetime,open,high,low,close,volume
2020-01-02,1.11715,1.12015,1.11335,1.11445,85234
2020-01-03,1.11445,1.11825,1.11215,1.11665,92156
2020-01-06,1.11665,1.12105,1.11485,1.11795,78945
...
```

**Requirements**:
- At least 500 bars of data (for indicator warm-up)
- Daily EURUSD data
- Sorted chronologically (oldest first)

### Golden File Generation

**Process**:
1. Use established library (TA-Lib, pandas-ta, or verified MQL5) to calculate indicators
2. Save results as CSV files in `tests/fixtures/golden/`
3. Include multiple parameter sets

**Example Golden File**: `tests/fixtures/golden/rsi_14.csv`
```csv
bar,rsi
0,50.0
1,50.0
...
14,67.234
15,65.891
```

## 6.2 Test Categories

### Unit Tests

**Scope**: Individual functions, primitives, indicators

**Coverage**:
- Each primitive with various inputs
- Each indicator (vectorized and stateful)
- Edge cases (empty data, single bar, division by zero)
- Parameter validation

**Example**:
```rust
#[test]
fn test_sma_edge_cases() {
    let sma = SMA::new(5);
    
    // Empty data
    let empty = Series::new("close", Vec::<f64>::new());
    assert!(sma.calculate_vectorized(&[IndicatorArg::Series(empty.into())]).is_err());
    
    // Single bar
    let single = Series::new("close", vec![100.0]);
    let result = sma.calculate_vectorized(&[IndicatorArg::Series(single.into())]).unwrap();
    // Should handle gracefully
    
    // Period larger than data
    let small = Series::new("close", vec![1.0, 2.0, 3.0]);
    let result = sma.calculate_vectorized(&[IndicatorArg::Series(small.into())]).unwrap();
    // Should return NaN or appropriate default
}
```

### Integration Tests

**Scope**: Complete workflows from data loading to code generation

**Coverage**:
- Full backtesting pipeline
- Strategy evaluation
- Metrics calculation
- Code generation

**Location**: `tests/integration_tests.rs`

### Verification Tests

**Scope**: Consistency checks between different modes

**Coverage**:
- Vectorized vs stateful
- Rust vs golden files
- Rust vs MQL5 (when available)

**Location**: `tests/indicator_verification.rs`

## 6.3 Continuous Testing Strategy

### Pre-commit Checks

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check

# Clippy (linting)
cargo clippy --all-targets --all-features -- -D warnings

# Unit tests
cargo test --lib

# Quick integration test
cargo test test_full_strategy_pipeline

echo "✓ All pre-commit checks passed"
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml

name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          
      - name: Cache cargo registry
        uses: actions/cache@v2
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Run tests
        run: cargo test --all-features --verbose
        
      - name: Run verification tests
        run: cargo test --test indicator_verification
        
      - name: Check code formatting
        run: cargo fmt -- --check
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

---

# 7. Performance Optimization

## 7.1 Profiling Strategy

### Identify Hotspots

**Tools**:
- `cargo-flamegraph`: Visual profiling
- `criterion`: Benchmarking
- `perf`: Linux performance tools

**Process**:
```bash
# Install profiling tools
cargo install flamegraph
cargo install cargo-criterion

# Profile a backtest
cargo flamegraph --bin tradebias -- backtest sample_data.csv

# Run benchmarks
cargo criterion
```

### Expected Hotspots

1. **Indicator calculation loops** (60-70% of time)
2. **Portfolio simulation** (15-20% of time)
3. **Polars expression evaluation** (10-15% of time)
4. **AST building/validation** (5% of time)

## 7.2 Optimization Techniques

### Vectorization with Polars

**Before (inefficient)**:
```rust
// Don't do this - row-by-row processing
let mut results = Vec::new();
for i in 0..data.height() {
    let close = data.column("close")?.f64()?.get(i)?;
    let sma = calculate_sma_single(close, 14);
    results.push(sma);
}
```

**After (efficient)**:
```rust
// Use Polars vectorized operations
let sma = data
    .column("close")?
    .rolling_mean(RollingOptionsFixedWindow {
        window_size: 14,
        ..Default::default()
    });
```

**Performance gain**: 10-100x faster

### Parallel Strategy Evaluation

**Implementation**:
```rust
use rayon::prelude::*;

pub fn evaluate_population(
    &self,
    population: &[AstNode],
    data: &DataFrame,
) -> Vec<StrategyResult> {
    population
        .par_iter()
        .map(|strategy| {
            self.backtester.run(strategy, data).unwrap()
        })
        .collect()
}
```

**Performance gain**: Near-linear scaling with CPU cores

### Indicator Caching

**Impact**: 10-100x speedup when multiple strategies use same indicators

**Implementation**: See Section 2.4

### Memory Pooling

**For frequent allocations**:
```rust
use typed_arena::Arena;

pub struct StrategyGenerator<'a> {
    ast_arena: &'a Arena<AstNode>,
}

impl<'a> StrategyGenerator<'a> {
    pub fn generate(&self) -> &'a AstNode {
        // Allocate in arena (fast)
        self.ast_arena.alloc(AstNode::Call { /* ... */ })
    }
}
```

**Performance gain**: 2-5x faster for AST-heavy operations

## 7.3 Benchmarking Suite

### Benchmark Definitions

**File**: `benches/indicators.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tradebias::functions::indicators::*;
use polars::prelude::*;

fn benchmark_rsi(c: &mut Criterion) {
    let data = generate_test_data(1000);
    let rsi = RSI::new(14);
    
    c.bench_function("rsi_vectorized", |b| {
        b.iter(|| {
            rsi.calculate_vectorized(black_box(&[
                IndicatorArg::Series(col("close")),
                IndicatorArg::Scalar(14.0),
            ]))
        })
    });
    
    c.bench_function("rsi_stateful", |b| {
        let mut state = rsi.init_state();
        let prices: Vec<f64> = data.column("close")
            .unwrap()
            .f64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        
        b.iter(|| {
            for &price in &prices {
                black_box(rsi.calculate_stateful(&[price], state.as_mut()));
            }
        })
    });
}

fn benchmark_backtest(c: &mut Criterion) {
    let data = generate_test_data(5000);
    let registry = FunctionRegistry::new(Default::default());
    let backtester = Backtester::new(registry);
    let strategy = create_test_strategy();
    
    c.bench_function("backtest_full", |b| {
        b.iter(|| {
            backtester.run(black_box(&strategy), black_box(&data))
        })
    });
}

criterion_group!(benches, benchmark_rsi, benchmark_backtest);
criterion_main!(benches);
```

**Run benchmarks**:
```bash
cargo bench
```

**Expected performance targets**:
- RSI vectorized (1000 bars): < 1ms
- RSI stateful (1000 bars): < 10ms
- Full backtest (5000 bars): < 100ms
- Evolution generation (100 strategies): < 5 seconds

---

# 8. AI Agent Implementation Order

## 8.1 Dependency-Ordered Task List

**For Gemini CLI or autonomous agents**

### Priority 1: Foundation (No Dependencies)

```markdown
TASK_001: Create project structure
  INPUT: None
  OUTPUT: Directory tree, Cargo.toml
  VERIFY: cargo check
  ESTIMATED_TIME: 30 minutes

TASK_002: Implement error types
  INPUT: None
  OUTPUT: src/error.rs
  VERIFY: cargo check
  ESTIMATED_TIME: 1 hour

TASK_003: Implement core types
  INPUT: None
  OUTPUT: src/types.rs
  VERIFY: cargo test types
  ESTIMATED_TIME: 2 hours

TASK_004: Implement trait definitions
  INPUT: None
  OUTPUT: src/functions/traits.rs
  VERIFY: cargo check
  ESTIMATED_TIME: 2 hours
```

### Priority 2: Primitives (Depends on P1)

```markdown
TASK_005: Implement MovingAverage primitive
  INPUT: traits.rs
  OUTPUT: src/functions/primitives.rs (partial)
  VERIFY: cargo test primitives::moving_average
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_004

TASK_006: Implement statistical primitives
  INPUT: traits.rs
  OUTPUT: src/functions/primitives.rs (continued)
  VERIFY: cargo test primitives::statistical
  ESTIMATED_TIME: 3 hours
  DEPENDENCIES: TASK_004

TASK_007: Implement arithmetic primitives
  INPUT: traits.rs
  OUTPUT: src/functions/primitives.rs (complete)
  VERIFY: cargo test primitives
  ESTIMATED_TIME: 2 hours
  DEPENDENCIES: TASK_004
```

### Priority 3: Core Indicators (Depends on P2)

```markdown
TASK_008: Implement RSI indicator
  INPUT: traits.rs, primitives.rs
  OUTPUT: src/functions/indicators/momentum.rs (partial)
  VERIFY: cargo test indicators::rsi
  ESTIMATED_TIME: 6 hours
  DEPENDENCIES: TASK_005, TASK_006

TASK_009: Implement SMA/EMA indicators
  INPUT: traits.rs, primitives.rs
  OUTPUT: src/functions/indicators/trend.rs (partial)
  VERIFY: cargo test indicators::moving_averages
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_005

TASK_010: Implement MACD indicator
  INPUT: traits.rs, primitives.rs, EMA
  OUTPUT: src/functions/indicators/trend.rs (continued)
  VERIFY: cargo test indicators::macd
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_009

TASK_011: Implement Bollinger Bands
  INPUT: traits.rs, primitives.rs
  OUTPUT: src/functions/indicators/volatility.rs (partial)
  VERIFY: cargo test indicators::bollinger
  ESTIMATED_TIME: 3 hours
  DEPENDENCIES: TASK_005, TASK_006

TASK_012: Implement ATR
  INPUT: traits.rs, primitives.rs
  OUTPUT: src/functions/indicators/volatility.rs (continued)
  VERIFY: cargo test indicators::atr
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_005, TASK_006

TASK_013: Implement Stochastic
  INPUT: traits.rs, primitives.rs
  OUTPUT: src/functions/indicators/momentum.rs (continued)
  VERIFY: cargo test indicators::stochastic
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_006

TASK_014: Implement remaining Tier 1 indicators
  INPUT: All previous indicators
  OUTPUT: Complete Tier 1 indicator set
  VERIFY: cargo test indicators::tier1
  ESTIMATED_TIME: 12 hours
  DEPENDENCIES: TASK_008..TASK_013
```

### Priority 4: Registry & Manifest (Depends on P3)

```markdown
TASK_015: Implement function registry
  INPUT: traits.rs, all primitives, Tier 1 indicators
  OUTPUT: src/functions/registry.rs
  VERIFY: cargo test registry
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_014

TASK_016: Implement indicator manifest
  INPUT: registry.rs
  OUTPUT: src/functions/manifest.rs
  VERIFY: cargo test manifest
  ESTIMATED_TIME: 3 hours
  DEPENDENCIES: TASK_015

TASK_017: Implement indicator cache
  INPUT: types.rs
  OUTPUT: src/data/cache.rs
  VERIFY: cargo test cache
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_003
```

### Priority 5: Backtesting Engine (Depends on P4)

```markdown
TASK_018: Implement expression builder
  INPUT: types.rs, registry.rs
  OUTPUT: src/engines/evaluation/expression.rs
  VERIFY: cargo test expression_builder
  ESTIMATED_TIME: 6 hours
  DEPENDENCIES: TASK_015

TASK_019: Implement portfolio simulator
  INPUT: types.rs
  OUTPUT: src/engines/evaluation/portfolio.rs
  VERIFY: cargo test portfolio
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_003

TASK_020: Implement backtester
  INPUT: expression.rs, portfolio.rs
  OUTPUT: src/engines/evaluation/backtester.rs
  VERIFY: cargo test backtester
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_018, TASK_019

TASK_021: Integration test - simple backtest
  INPUT: backtester.rs, sample_data.csv
  OUTPUT: tests/integration_tests.rs (partial)
  VERIFY: cargo test test_simple_backtest
  ESTIMATED_TIME: 2 hours
  DEPENDENCIES: TASK_020
```

### Priority 6: Metrics Engine (Depends on P5)

```markdown
TASK_022: Implement core metrics
  INPUT: types.rs
  OUTPUT: src/engines/metrics/*.rs
  VERIFY: cargo test metrics
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_003

TASK_023: Implement metrics engine with dependency resolution
  INPUT: metrics/*.rs
  OUTPUT: src/engines/metrics/engine.rs
  VERIFY: cargo test metrics_engine
  ESTIMATED_TIME: 6 hours
  DEPENDENCIES: TASK_022

TASK_024: Integration test - full backtest with metrics
  INPUT: backtester.rs, metrics_engine.rs
  OUTPUT: tests/integration_tests.rs (continued)
  VERIFY: cargo test test_backtest_with_metrics
  ESTIMATED_TIME: 2 hours
  DEPENDENCIES: TASK_020, TASK_023
```

### Priority 7: Code Generation (Depends on P6)

```markdown
TASK_025: Implement MQL5 indicator library generator
  INPUT: registry.rs, all indicators
  OUTPUT: src/codegen/mql5_indicators.rs
  VERIFY: Inspect generated MQH file
  ESTIMATED_TIME: 12 hours
  DEPENDENCIES: TASK_015

TASK_026: Implement MQL5 EA generator
  INPUT: types.rs, registry.rs, mql5_indicators.rs
  OUTPUT: src/codegen/mql5_ea.rs
  VERIFY: Inspect generated EA file
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_025

TASK_027: Manual MQL5 verification
  INPUT: Generated MQH and EA files
  OUTPUT: Verification report
  VERIFY: Run in MQL5, compare outputs
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_026
```

### Priority 8: Verification Suite (Depends on P7)

```markdown
TASK_028: Create golden files for all indicators
  INPUT: sample_data.csv, reference implementations
  OUTPUT: tests/fixtures/golden/*.csv
  VERIFY: Manual inspection
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_014

TASK_029: Implement golden file tests
  INPUT: golden files, indicators
  OUTPUT: tests/indicator_verification.rs (partial)
  VERIFY: cargo test golden_files
  ESTIMATED_TIME: 6 hours
  DEPENDENCIES: TASK_028

TASK_030: Implement consistency tests (vectorized vs stateful)
  INPUT: all indicators
  OUTPUT: tests/indicator_verification.rs (continued)
  VERIFY: cargo test consistency
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_014

TASK_031: Document all test results
  INPUT: All test outputs
  OUTPUT: TEST_RESULTS.md
  VERIFY: Manual review
  ESTIMATED_TIME: 2 hours
  DEPENDENCIES: TASK_029, TASK_030
```

### Priority 9: Remaining Components (Can be done in parallel)

```markdown
TASK_032: Implement data connectors
  INPUT: types.rs
  OUTPUT: src/data/connectors/*.rs
  VERIFY: cargo test connectors
  ESTIMATED_TIME: 6 hours
  DEPENDENCIES: TASK_003

TASK_033: Implement config management
  INPUT: types.rs
  OUTPUT: src/config/*.rs
  VERIFY: cargo test config
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_003

TASK_034: Implement Tier 2 indicators
  INPUT: primitives.rs, Tier 1 indicators
  OUTPUT: Complete Tier 2 set
  VERIFY: cargo test indicators::tier2
  ESTIMATED_TIME: 20 hours
  DEPENDENCIES: TASK_014

TASK_035: Implement genetic algorithm
  INPUT: types.rs, registry.rs, backtester.rs
  OUTPUT: src/engines/generation/*.rs
  VERIFY: cargo test generation
  ESTIMATED_TIME: 16 hours
  DEPENDENCIES: TASK_020

TASK_036: Implement ML pipeline
  INPUT: types.rs, backtester.rs
  OUTPUT: src/ml/*/.rs
  VERIFY: cargo test ml
  ESTIMATED_TIME: 24 hours
  DEPENDENCIES: TASK_020
```

### Priority 10: Final Integration

```markdown
TASK_037: Integrate with egui UI
  INPUT: All engines, public API
  OUTPUT: Updated ui/ directory
  VERIFY: Manual UI testing
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_035, TASK_036

TASK_038: Performance optimization
  INPUT: Complete codebase
  OUTPUT: Optimized implementations
  VERIFY: cargo bench
  ESTIMATED_TIME: 16 hours
  DEPENDENCIES: TASK_037

TASK_039: Documentation
  INPUT: Complete codebase
  OUTPUT: README.md, API docs
  VERIFY: cargo doc
  ESTIMATED_TIME: 8 hours
  DEPENDENCIES: TASK_037

TASK_040: Release build and testing
  INPUT: Complete codebase
  OUTPUT: Optimized binary
  VERIFY: Full system test
  ESTIMATED_TIME: 4 hours
  DEPENDENCIES: TASK_038, TASK_039
```

## 8.2 Total Time Estimate

**By Priority Level**:
- P1 (Foundation): 5.5 hours
- P2 (Primitives): 9 hours
- P3 (Core Indicators): 37 hours
- P4 (Registry): 11 hours
- P5 (Backtesting): 20 hours
- P6 (Metrics): 16 hours
- P7 (Code Gen): 24 hours
- P8 (Verification): 20 hours
- P9 (Remaining): 70 hours
- P10 (Final): 36 hours

**Total Estimated Time**: ~248.5 hours (≈6 weeks at 40 hours/week)

**Critical Path**: P1 → P2 → P3 → P4 → P5 → P6 → P7 → P8 → P10

## 8.3 AI Agent Command Structure

### Task Specification Format

Each task given to AI agent should follow this structure:

```markdown
### TASK ID: TASK_XXX

**Objective**: [One-sentence goal]

**Inputs**:
- File: [path/to/input.rs]
- Specification: [Section X.Y of this document]
- Dependencies: [TASK_001, TASK_002]

**Outputs**:
- File: [path/to/output.rs]
- Tests: [path/to/test.rs]

**Implementation Steps**:
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Mathematical Specification** (if applicable):
```
[Formula or algorithm]
```

**Verification Command**:
```bash
cargo test [test_name]
```

**Success Criteria**:
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Clippy passes
- [ ] Code formatted with rustfmt

**Estimated Time**: [X hours]
```

### Example Task for AI Agent

```markdown
### TASK ID: TASK_008

**Objective**: Implement RSI (Relative Strength Index) indicator with both vectorized and stateful modes

**Inputs**:
- File: src/functions/traits.rs
- File: src/functions/primitives.rs
- Specification: Section 4.2 (RSI Complete Dual Implementation)
- Dependencies: TASK_005, TASK_006

**Outputs**:
- File: src/functions/indicators/momentum.rs
- Tests: Add to tests/indicators.rs

**Implementation Steps**:
1. Create `RSI` struct with `period` field
2. Create `RSIState` struct with: period, gains buffer, losses buffer, avg_gain, avg_loss, prev_close
3. Implement `Indicator` trait for RSI
4. Implement `calculate_vectorized()`:
   - Calculate price deltas using diff(1)
   - Separate gains (max(delta, 0)) and losses (max(-delta, 0))
   - Apply Wilder's smoothing (SMMA) to gains and losses
   - Calculate RS = avg_gain / avg_loss
   - Calculate RSI = 100 - (100 / (1 + RS))
5. Implement `calculate_stateful()`:
   - Maintain VecDeque buffers for gains and losses
   - Calculate average using Wilder's smoothing algorithm
   - Handle first period (use simple average)
   - Subsequent periods use: avg[i] = (avg[i-1] * (period-1) + value[i]) / period
6. Implement `init_state()` returning boxed RSIState
7. Implement `generate_mql5()` returning "TB_RSI(close, period)"

**Mathematical Specification**:
```
delta[i] = close[i] - close[i-1]
gain[i] = max(delta[i], 0)
loss[i] = max(-delta[i], 0)

First avg_gain = SMA(gains, period)
First avg_loss = SMA(losses, period)

Subsequent:
avg_gain[i] = (avg_gain[i-1] * (period-1) + gain[i]) / period
avg_loss[i] = (avg_loss[i-1] * (period-1) + loss[i]) / period

RS = avg_gain / avg_loss
RSI = 100 - (100 / (1 + RS))
```

**Verification Command**:
```bash
cargo test indicators::rsi
cargo test indicators::rsi::test_dual_implementation
```

**Success Criteria**:
- [x] RSI struct implements Indicator trait
- [x] Vectorized mode returns correct Polars expression
- [x] Stateful mode calculates bar-by-bar correctly
- [x] Both modes produce identical results (within 1e-6 tolerance)
- [x] Handles edge cases (not enough data, zero division)
- [x] All tests pass
- [x] No clippy warnings

**Estimated Time**: 6 hours
```

---

# 9. Deployment & Distribution

## 9.1 Build Configuration

### Development Build

```bash
cargo build
```

**Features**:
- Debug symbols
- Faster compilation
- Runtime checks enabled

### Release Build

```bash
cargo build --release
```

**Optimizations**:
- LTO (Link-Time Optimization)
- Strip symbols
- Optimize for size or speed

**Cargo.toml optimizations**:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

### Target Platforms

**Primary targets**:
- Windows x86_64
- macOS x86_64 / ARM64
- Linux x86_64

**Cross-compilation**:
```bash
# Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu

# macOS from Linux
cargo build --release --target x86_64-apple-darwin

# Linux from macOS
cargo build --release --target x86_64-unknown-linux-gnu
```

## 9.2 Binary Distribution

### Single-Binary Packaging

**Structure**:
```
TradeBias-v1.0.0/
├── tradebias(.exe)              # Main executable
├── TradeBias_Indicators.mqh     # MQL5 indicator library (bundled)
├── README.txt
├── LICENSE.txt
└── sample_data.csv              # Example data
```

**Size target**: 30-50 MB (with embedded UI)

### Installation Process

**User steps**:
1. Download `TradeBias-v1.0.0.zip`
2. Extract to desired location
3. Run `tradebias.exe` (or `./tradebias` on Unix)
4. UI opens immediately

**No dependencies required** (statically linked)

## 9.3 MQL5 Deployment Workflow

### Generated Files

**For user's MetaTrader**:
1. `{StrategyName}_EA.mq5` - Expert Advisor
2. `TradeBias_Indicators.mqh` - Indicator library

**User workflow**:
1. User designs strategy in TradeBias app
2. User clicks "Generate MQL5 EA"
3. App saves two files to desktop
4. User copies both files to `C:\Program Files\MetaTrader 5\MQL5\Experts\`
5. User compiles EA in MetaEditor
6. User attaches EA to chart

### Template EA Structure

```cpp
//+------------------------------------------------------------------+
//| Strategy_{HASH}.mq5                                              |
//| Generated by TradeBias v1.0.0                                    |
//| DO NOT MODIFY - Regenerate if changes needed                    |
//+------------------------------------------------------------------+
#property strict
#property copyright "TradeBias"
#property version   "1.00"

#include "TradeBias_Indicators.mqh"

//--- Input parameters
input double LotSize = 0.1;
input double StopLoss = 50;
input double TakeProfit = 100;

//--- Global variables
int position_ticket = 0;

//+------------------------------------------------------------------+
//| Expert initialization function                                   |
//+------------------------------------------------------------------+
int OnInit() {
    Print("Strategy initialized: {STRATEGY_NAME}");
    return(INIT_SUCCEEDED);
}

//+------------------------------------------------------------------+
//| Expert tick function                                             |
//+------------------------------------------------------------------+
void OnTick() {
    // Get current data
    double close[], high[], low[];
    ArraySetAsSeries(close, true);
    ArraySetAsSeries(high, true);
    ArraySetAsSeries(low, true);
    
    CopyClose(_Symbol, _Period, 0, 100, close);
    CopyHigh(_Symbol, _Period, 0, 100, high);
    CopyLow(_Symbol, _Period, 0, 100, low);
    
    // Check entry condition
    if(CheckEntryCondition(close, high, low)) {
        OpenPosition();
    }
}

//+------------------------------------------------------------------+
//| Entry condition (generated from strategy AST)                   |
//+------------------------------------------------------------------+
bool CheckEntryCondition(const double &close[], const double &high[], const double &low[]) {
    // GENERATED CODE BLOCK START
    {GENERATED_CONDITION_CODE}
    // GENERATED CODE BLOCK END
}

//+------------------------------------------------------------------+
//| Open position                                                    |
//+------------------------------------------------------------------+
void OpenPosition() {
    MqlTradeRequest request;
    MqlTradeResult result;
    ZeroMemory(request);
    ZeroMemory(result);
    
    request.action = TRADE_ACTION_DEAL;
    request.symbol = _Symbol;
    request.volume = LotSize;
    request.type = ORDER_TYPE_BUY;
    request.price = SymbolInfoDouble(_Symbol, SYMBOL_ASK);
    request.sl = request.price - StopLoss * _Point;
    request.tp = request.price + TakeProfit * _Point;
    request.deviation = 5;
    request.magic = 123456;
    request.comment = "TradeBias Strategy";
    
    if(!OrderSend(request, result)) {
        Print("Order send failed: ", GetLastError());
    } else {
        position_ticket = result.order;
        Print("Position opened: Ticket = ", position_ticket);
    }
}
```

---

# 10. Maintenance & Extensibility

## 10.1 Adding New Indicators

### Step-by-Step Process

**1. Create indicator struct**:
```rust
// src/functions/indicators/momentum.rs

pub struct MyNewIndicator {
    period: usize,
    threshold: f64,
}
```

**2. Create state struct**:
```rust
pub struct MyNewIndicatorState {
    period: usize,
    buffer: VecDeque<f64>,
    // ... other state variables
}
```

**3. Implement Indicator trait**:
```rust
impl Indicator for MyNewIndicator {
    fn alias(&self) -> &'static str { "MyIndicator" }
    fn ui_name(&self) -> &'static str { "My New Indicator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((0.0, 100.0)) }
    fn arity(&self) -> usize { 2 }
    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }
    
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        // Implement using Polars operations
        todo!()
    }
    
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64> {
        // Implement bar-by-bar calculation
        todo!()
    }
    
    fn init_state(&self) -> Box<dyn Any> {
        Box::new(MyNewIndicatorState {
            period: self.period,
            buffer: VecDeque::new(),
        })
    }
    
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("TB_MyIndicator({}, {})", args[0], args[1])
    }
}
```

**4. Add to registry**:
```rust
// src/functions/registry.rs

impl FunctionRegistry {
    fn register_indicators(&mut self) {
        // ... existing indicators
        
        self.register_indicator(
            "MyIndicator",
            Arc::new(MyNewIndicator { period: 14, threshold: 50.0 })
        );
    }
}
```

**5. Add to manifest (if Tier 1 or 2)**:
```rust
// src/functions/manifest.rs

const TIER2_INDICATORS: &[&str] = &[
    // ... existing
    "MyIndicator",
];
```

**6. Write tests**:
```rust
#[test]
fn test_my_indicator() {
    let data = load_sample_data();
    let indicator = MyNewIndicator::new(14, 50.0);
    
    // Test vectorized
    let vec_result = /* ... */;
    
    // Test stateful
    let state_result = /* ... */;
    
    // Compare
    assert_consistency(&vec_result, &state_result);
}
```

**7. Add MQL5 implementation**:
```rust
// src/codegen/mql5_indicators.rs

fn generate_my_indicator(&self) -> String {
    r#"
double TB_MyIndicator(const double &prices[], int period, int shift) {
    // Implement matching algorithm
    return 0.0;
}
"#.to_string()
}
```

**Time estimate**: 4-8 hours for a new indicator

## 10.2 Adding New Metrics

### Process

**1. Implement metric calculator**:
```rust
// src/engines/metrics/custom.rs

pub struct MyCustomMetric;

impl MetricCalculator for MyCustomMetric {
    fn id(&self) -> &'static str { "my_custom_metric" }
    fn display_name(&self) -> &'static str { "My Custom Metric" }
    
    fn dependencies(&self) -> Vec<&'static str> {
        vec!["equity_curve", "total_trades"]
    }
    
    fn calculate(&self, context: &MetricContext) -> Result<f64> {
        let equity = context.equity_curve;
        let trades = context.cache.get("total_trades").unwrap();
        
        // Calculate metric
        let result = /* ... */;
        
        Ok(result)
    }
}
```

**2. Register in engine**:
```rust
// src/engines/metrics/engine.rs

impl MetricsEngine {
    pub fn new() -> Self {
        let mut calculators: HashMap<String, Box<dyn MetricCalculator>> = HashMap::new();
        
        // ... existing metrics
        calculators.insert("my_custom_metric".into(), Box::new(MyCustomMetric));
        
        Self { calculators }
    }
}
```

**Time estimate**: 1-2 hours

## 10.3 Version Control Strategy

### Git Workflow

**Branches**:
- `main`: Production-ready code
- `develop`: Integration branch
- `feature/*`: Feature branches
- `hotfix/*`: Critical fixes

**Commit conventions**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `perf`: Performance improvement
- `refactor`: Code refactoring
- `test`: Test additions
- `docs`: Documentation

**Example**:
```
feat(indicators): Add RSI implementation

Implements RSI with dual calculation modes (vectorized and stateful).
Includes comprehensive tests and MQL5 code generation.

Closes #42
```

---

# 11. Appendix

## 11.1 Complete Cargo.toml

```toml
[package]
name = "tradebias"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Algorithmic trading strategy generation and backtesting platform"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/tradebias"

[dependencies]
# Data processing
polars = { version = "0.36", features = ["lazy", "csv", "json", "dtype-datetime", "rolling_window"] }
arrow = "50"

# Numerical computing
ndarray = "0.15"
nalgebra = "0.32"
statrs = "0.16"

# Randomness (for GA)
rand = "0.8"
rand_distr = "0.4"

# Machine learning
smartcore = { version = "0.3", features = ["serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Configuration
config = "0.13"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time handling
chrono = "0.4"

# Frontend (egui)
egui = "0.25"
eframe = "0.25"

# Parallelization
rayon = "1.8"

# Utilities
once_cell = "1.19"
typed-arena = "2.0"

# HTTP (optional, for Supabase)
reqwest = { version = "0.11", features = ["json"], optional = true }
tokio = { version = "1", features = ["full"], optional = true }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.4"
tempfile = "3"

[features]
default = ["api"]
api = ["reqwest", "tokio"]

[[bench]]
name = "indicators"
harness = false

[[bench]]
name = "backtest"
harness = false

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
opt-level = 1  # Some optimization in dev for better performance

[profile.test]
opt-level = 2  # Optimize tests
```

## 11.2 Key Dependencies Explained

| Crate | Version | Purpose | Why This Version |
|-------|---------|---------|------------------|
| `polars` | 0.36 | Vectorized data operations | Latest stable, excellent performance |
| `smartcore` | 0.3 | ML models | Best Rust ML library for our needs |
| `serde` | 1.0 | Serialization | Industry standard |
| `rayon` | 1.8 | Parallel processing | Simple parallelization API |
| `egui` | 0.25 | UI framework | Latest stable, immediate mode |
| `anyhow` | 1.0 | Error handling | Simple error propagation |
| `thiserror` | 1.0 | Custom errors | Derive macro for error types |
| `chrono` | 0.4 | Date/time handling | Standard datetime library |

## 11.3 Mathematical Reference

### Common Formulas

**Simple Moving Average (SMA)**:
```
SMA[i] = (Σ price[i-j] for j=0 to N-1) / N
```

**Exponential Moving Average (EMA)**:
```
α = 2 / (N + 1)
EMA[0] = price[0]
EMA[i] = α * price[i] + (1 - α) * EMA[i-1]
```

**Relative Strength Index (RSI)**:
```
delta[i] = price[i] - price[i-1]
gain[i] = max(delta[i], 0)
loss[i] = max(-delta[i], 0)

avg_gain = SMMA(gains, N)
avg_loss = SMMA(losses, N)

RS = avg_gain / avg_loss
RSI = 100 - (100 / (1 + RS))
```

**Bollinger Bands**:
```
Middle = SMA(close, N)
Upper = Middle + k * σ(close, N)
Lower = Middle - k * σ(close, N)
```

**Average True Range (ATR)**:
```
TR = max(high - low, |high - prev_close|, |low - prev_close|)
ATR = SMMA(TR, N)
```

**Sharpe Ratio**:
```
Returns[i] = (equity[i] - equity[i-1]) / equity[i-1]
Mean = Σ Returns / n
StdDev = sqrt(Σ (Returns - Mean)² / (n-1))
Sharpe = (Mean - RiskFreeRate) / StdDev * sqrt(PeriodsPerYear)
```

## 11.4 Glossary

**AST (Abstract Syntax Tree)**: Tree representation of strategy logic

**Backtesting**: Simulating strategy on historical data

**Composed Indicator**: Indicator built from primitives (e.g., RSI, MACD)

**EA (Expert Advisor)**: MQL5 automated trading program

**Equity Curve**: Chart showing account balance over time

**Genetic Algorithm**: Evolutionary optimization technique

**Hall of Fame**: Best strategies from evolution

**Indicator**: Technical analysis calculation (RSI, SMA, etc.)

**MQL5**: MetaTrader 5 programming language

**Polars**: Rust DataFrame library for high-performance data operations

**Primitive**: Basic building block (SMA, Max, Min, Add, etc.)

**Semantic Mapper**: Generates type-valid strategies from genomes

**Stateful**: Bar-by-bar calculation maintaining state

**Vectorized**: Batch calculation over entire series

**Walk-Forward Optimization**: Out-of-sample validation technique

## 11.5 Resources

### Documentation

- **Polars**: https://pola-rs.github.io/polars-book/
- **egui**: https://docs.rs/egui/
- **MQL5**: https://www.mql5.com/en/docs
- **Rust**: https://doc.rust-lang.org/book/

### Testing Tools

- **Criterion**: https://bheisler.github.io/criterion.rs/
- **Cargo test**: https://doc.rust-lang.org/cargo/commands/cargo-test.html

### Trading References

- **Technical Analysis**: "Technical Analysis of the Financial Markets" by John Murphy
- **Algorithmic Trading**: "Advances in Financial Machine Learning" by Marcos López de Prado

---

# Document Complete

**Version**: 1.0
**Date**: 2025-11-11
**Total Pages**: 60+
**Total Sections**: 11
**Total Tasks**: 40
**Estimated Migration Time**: ~250 hours (6 weeks)

This document provides complete specifications for migrating TradeBias from Python to Rust with custom indicator implementations ensuring mathematical consistency between backtesting and live trading.

**Next Steps**:
1. Review and approve architecture
2. Begin Phase 1 implementation (Foundation)
3. Validate indicator calculations against sample data
4. Proceed through phases sequentially
5. Generate and test MQL5 code
6. Deploy production version

**For Questions or Clarifications**: Reference section numbers and task IDs for specific guidance.