# 01 - Project Architecture

## Goal
Set up the complete project structure with all necessary directories and understand the module dependency rules.

## Prerequisites
- None (this is the first implementation step)
- Have Rust installed with Cargo

## What You'll Create
1. Complete directory structure for the TradeBias project
2. Empty module files with proper exports
3. Basic Cargo.toml with dependencies

## High-Level System Design

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

## Complete Directory Structure

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

## Module Dependency Graph

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

## Critical Dependency Rules

**IMPORTANT: Follow these rules strictly to avoid circular dependencies**

1. **NO circular dependencies allowed**
2. **Data layer** only depends on core types (`error.rs`, `types.rs`, `config/`)
3. **Functions layer** only depends on core types
4. **Engines layer** depends on functions + data
5. **UI layer** only calls public API in `lib.rs`
6. **Code Generator** depends on functions layer
7. **ML Pipeline** depends on functions + data

## Implementation Steps

### Step 1: Create Directory Structure

```bash
# From project root
mkdir -p src/config
mkdir -p src/data/connectors
mkdir -p src/functions/indicators
mkdir -p src/engines/generation
mkdir -p src/engines/evaluation
mkdir -p src/engines/metrics
mkdir -p src/engines/validation
mkdir -p src/ml/features
mkdir -p src/ml/labeling
mkdir -p src/ml/models
mkdir -p src/codegen
mkdir -p src/utils
mkdir -p tests/fixtures
```

### Step 2: Create Basic Cargo.toml

```toml
[package]
name = "tradebias"
version = "0.1.0"
edition = "2021"

[dependencies]
# Data processing
polars = { version = "0.43", features = ["lazy", "temporal", "dtype-full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async runtime (for Supabase connector)
tokio = { version = "1.0", features = ["full"] }

# Config management
config = "0.14"

# Hashing (for cache keys)
blake3 = "1.5"

# Date/time
chrono = "0.4"

# Logging
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
approx = "0.5"
criterion = "0.5"

[[bench]]
name = "indicator_bench"
harness = false
```

### Step 3: Create Module Files with Exports

Create each file with basic module exports. Here's the pattern for each `mod.rs`:

**src/lib.rs**:
```rust
// Public API
pub mod config;
pub mod data;
pub mod functions;
pub mod engines;
pub mod ml;
pub mod codegen;
pub mod utils;

// Re-export commonly used types
pub mod error;
pub mod types;

pub use error::TradeBiasError;
pub use types::*;

// Public API functions (to be implemented later)
pub fn run_evolution() -> Result<(), TradeBiasError> {
    todo!("To be implemented in engines module")
}

pub fn run_backtest() -> Result<(), TradeBiasError> {
    todo!("To be implemented in engines/evaluation")
}

pub fn train_ml_model() -> Result<(), TradeBiasError> {
    todo!("To be implemented in ml module")
}

pub fn generate_mql5_ea() -> Result<String, TradeBiasError> {
    todo!("To be implemented in codegen module")
}
```

**src/config/mod.rs**:
```rust
pub mod manager;
pub mod schema;

pub use manager::ConfigManager;
pub use schema::*;
```

**src/data/mod.rs**:
```rust
pub mod types;
pub mod connectors;
pub mod cache;

pub use types::*;
pub use cache::IndicatorCache;
```

**src/data/connectors/mod.rs**:
```rust
pub mod csv;
pub mod supabase;

pub use csv::CsvConnector;
pub use supabase::SupabaseConnector;
```

**src/functions/mod.rs**:
```rust
pub mod traits;
pub mod primitives;
pub mod indicators;
pub mod manifest;
pub mod registry;

pub use traits::*;
pub use primitives::*;
pub use manifest::IndicatorManifest;
pub use registry::FunctionRegistry;
```

**src/functions/indicators/mod.rs**:
```rust
pub mod trend;
pub mod momentum;
pub mod volatility;
pub mod volume;

pub use trend::*;
pub use momentum::*;
pub use volatility::*;
pub use volume::*;
```

**src/engines/mod.rs**:
```rust
pub mod generation;
pub mod evaluation;
pub mod metrics;
pub mod validation;

pub use generation::*;
pub use evaluation::*;
pub use metrics::*;
pub use validation::*;
```

**src/engines/generation/mod.rs**:
```rust
pub mod ast;
pub mod genome;
pub mod semantic_mapper;
pub mod evaluator;
pub mod operators;

pub use ast::*;
pub use genome::*;
```

**src/engines/evaluation/mod.rs**:
```rust
pub mod backtester;
pub mod portfolio;
pub mod expression;

pub use backtester::Backtester;
pub use portfolio::Portfolio;
pub use expression::ExpressionBuilder;
```

**src/engines/metrics/mod.rs**:
```rust
pub mod engine;
pub mod profitability;
pub mod risk;
pub mod returns;

pub use engine::MetricsEngine;
```

**src/engines/validation/mod.rs**:
```rust
pub mod semantic;

pub use semantic::SemanticValidator;
```

**src/ml/mod.rs**:
```rust
pub mod features;
pub mod labeling;
pub mod models;

pub use features::*;
pub use labeling::*;
pub use models::*;
```

**src/ml/features/mod.rs**:
```rust
pub mod engineer;

pub use engineer::FeatureEngineer;
```

**src/ml/labeling/mod.rs**:
```rust
pub mod triple_barrier;

pub use triple_barrier::TripleBarrierLabeling;
```

**src/ml/models/mod.rs**:
```rust
pub mod base;
pub mod ensemble;

pub use base::*;
pub use ensemble::*;
```

**src/codegen/mod.rs**:
```rust
pub mod mql5_ea;
pub mod mql5_indicators;

pub use mql5_ea::EAGenerator;
pub use mql5_indicators::IndicatorLibraryGenerator;
```

**src/utils/mod.rs**:
```rust
pub mod ast_converter;
pub mod metadata;

pub use ast_converter::AstConverter;
pub use metadata::IndicatorMetadata;
```

### Step 4: Create Empty Implementation Files

For each leaf file (non-`mod.rs`), create with a basic structure:

**Example: src/config/manager.rs**:
```rust
use crate::error::TradeBiasError;

pub struct ConfigManager {
    // To be implemented
}

impl ConfigManager {
    pub fn new() -> Result<Self, TradeBiasError> {
        todo!("Implementation in later phase")
    }
}
```

**Example: src/data/cache.rs**:
```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use polars::prelude::*;

pub struct IndicatorCache {
    cache: Arc<RwLock<HashMap<CacheKey, CachedResult>>>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    pub indicator: String,
    pub params: Vec<i32>,
    pub data_hash: u64,
}

pub struct CachedResult {
    pub result: Series,
    pub size_bytes: usize,
}

impl IndicatorCache {
    pub fn new() -> Self {
        todo!("Implementation in Phase 4")
    }
}
```

Repeat this pattern for all files, creating the structure but leaving implementations as `todo!()`.

### Step 5: Verify Project Compiles

```bash
cargo check
```

You should see warnings about unused code and `todo!()` macros, but no errors.

## Verification

After completing this phase, verify:

1. **Directory structure is complete**:
   ```bash
   ls -R src/
   ```
   Should show all directories from the structure above.

2. **All mod.rs files exist**:
   ```bash
   find src -name "mod.rs"
   ```
   Should list at least 10 `mod.rs` files.

3. **Project compiles without errors**:
   ```bash
   cargo check
   ```
   Exit code should be 0 (warnings are OK).

4. **Dependency graph is clear**:
   - `functions/` should NOT import from `engines/`
   - `data/` should NOT import from `functions/` or `engines/`
   - `engines/` CAN import from `functions/` and `data/`

## Common Issues

### Issue: Circular dependency error
**Solution**: Check import statements. Make sure lower layers (data, functions) don't import from higher layers (engines).

### Issue: Module not found
**Solution**: Make sure you created the corresponding `mod.rs` file and added the module declaration.

### Issue: Cargo.toml dependency error
**Solution**: Run `cargo update` to refresh the dependency tree.

## Next Steps

Proceed to **[02-type-system.md](./02-type-system.md)** to implement core types, traits, and error handling.
