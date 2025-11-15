# Phase 2: Core Features Implementation

**Goal**: Complete all partially implemented core features and implement missing critical systems

**Priority**: HIGH - Required for basic functionality

**Prerequisites**: Phase 1 must be complete (project compiles)

## Task Breakdown

### 1. Configuration System Implementation (CRITICAL)

**Status**: NOT IMPLEMENTED - `src/config/mod.rs` is empty
**Spec**: `docs/ai-implementation/17-configuration-system.md`

#### 1.1 Create Configuration Traits
**File**: `src/config/traits.rs` (NEW)

```rust
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

/// Trait for configuration sections
pub trait ConfigSection: Serialize + for<'de> Deserialize<'de> + Default + Clone {
    fn section_name() -> &'static str;
    fn validate(&self) -> Result<(), TradebiasError>;
    fn to_manifest(&self) -> ConfigManifest;
}

/// Configuration manifest for UI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManifest {
    pub section: String,
    pub fields: Vec<FieldManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldManifest {
    pub name: String,
    pub field_type: String,
    pub default: serde_json::Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub description: String,
}
```

#### 1.2 Create Evolution Settings
**File**: `src/config/evolution.rs` (NEW)

```rust
use super::traits::{ConfigSection, ConfigManifest, FieldManifest};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub selection_method: SelectionMethod,
    pub elitism_count: usize,
    pub max_tree_depth: usize,
    pub tournament_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    Tournament,
    Roulette,
    Rank,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 500,
            num_generations: 100,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            selection_method: SelectionMethod::Tournament,
            elitism_count: 10,
            max_tree_depth: 12,
            tournament_size: 7,
        }
    }
}

impl ConfigSection for EvolutionConfig {
    fn section_name() -> &'static str {
        "evolution"
    }

    fn validate(&self) -> Result<(), TradebiasError> {
        if self.population_size < 10 {
            return Err(TradebiasError::Configuration(
                "Population size must be at least 10".to_string()
            ));
        }
        if self.mutation_rate < 0.0 || self.mutation_rate > 1.0 {
            return Err(TradebiasError::Configuration(
                "Mutation rate must be between 0 and 1".to_string()
            ));
        }
        if self.crossover_rate < 0.0 || self.crossover_rate > 1.0 {
            return Err(TradebiasError::Configuration(
                "Crossover rate must be between 0 and 1".to_string()
            ));
        }
        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        ConfigManifest {
            section: "Evolution".to_string(),
            fields: vec![
                FieldManifest {
                    name: "population_size".to_string(),
                    field_type: "integer".to_string(),
                    default: serde_json::json!(500),
                    min: Some(10.0),
                    max: Some(10000.0),
                    description: "Number of strategies in population".to_string(),
                },
                // ... add all other fields
            ],
        }
    }
}
```

#### 1.3 Create Backtesting Settings
**File**: `src/config/backtesting.rs` (NEW)

```rust
use super::traits::{ConfigSection, ConfigManifest};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestingConfig {
    pub validation_method: ValidationMethod,
    pub train_test_split: f64,
    pub num_folds: usize,
    pub initial_capital: f64,
    pub commission: f64,
    pub slippage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationMethod {
    Simple,
    WalkForward { anchored: bool },
    KFold,
}

impl Default for BacktestingConfig {
    fn default() -> Self {
        Self {
            validation_method: ValidationMethod::WalkForward { anchored: false },
            train_test_split: 0.7,
            num_folds: 5,
            initial_capital: 10000.0,
            commission: 0.001,
            slippage: 0.0005,
        }
    }
}

impl ConfigSection for BacktestingConfig {
    fn section_name() -> &'static str {
        "backtesting"
    }

    fn validate(&self) -> Result<(), TradebiasError> {
        if self.train_test_split <= 0.0 || self.train_test_split >= 1.0 {
            return Err(TradebiasError::Configuration(
                "Train/test split must be between 0 and 1".to_string()
            ));
        }
        if self.initial_capital <= 0.0 {
            return Err(TradebiasError::Configuration(
                "Initial capital must be positive".to_string()
            ));
        }
        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        // Similar to EvolutionConfig
        todo!()
    }
}
```

#### 1.4 Create Trade Management Settings
**File**: `src/config/trade_management.rs` (NEW)

```rust
use super::traits::{ConfigSection, ConfigManifest};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeManagementConfig {
    pub stop_loss: StopLossConfig,
    pub take_profit: TakeProfitConfig,
    pub position_sizing: PositionSizing,
    pub max_positions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopLossConfig {
    Fixed { percent: f64 },
    ATR { multiplier: f64, period: usize },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TakeProfitConfig {
    Fixed { percent: f64 },
    RiskReward { ratio: f64 },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizing {
    Fixed { size: f64 },
    Percent { percent: f64 },
    Kelly { fraction: f64 },
}

impl Default for TradeManagementConfig {
    fn default() -> Self {
        Self {
            stop_loss: StopLossConfig::ATR { multiplier: 2.0, period: 14 },
            take_profit: TakeProfitConfig::RiskReward { ratio: 2.0 },
            position_sizing: PositionSizing::Percent { percent: 0.02 },
            max_positions: 5,
        }
    }
}

impl ConfigSection for TradeManagementConfig {
    fn section_name() -> &'static str {
        "trade_management"
    }

    fn validate(&self) -> Result<(), TradebiasError> {
        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        todo!()
    }
}
```

#### 1.5 Create ML Settings
**File**: `src/config/ml.rs` (NEW)

```rust
use super::traits::{ConfigSection, ConfigManifest};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub feature_engineering: FeatureConfig,
    pub labeling: LabelingConfig,
    pub model_type: ModelType,
    pub training: TrainingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub include_returns: bool,
    pub include_volatility: bool,
    pub include_volume: bool,
    pub fractal_dimension: bool,
    pub hurst_exponent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelingConfig {
    pub method: LabelingMethod,
    pub barrier_width: f64,
    pub min_return: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelingMethod {
    TripleBarrier,
    FixedTime,
    FixedReturn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    RandomForest,
    GradientBoosting,
    NeuralNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub test_size: f64,
    pub cv_folds: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            feature_engineering: FeatureConfig {
                include_returns: true,
                include_volatility: true,
                include_volume: true,
                fractal_dimension: false,
                hurst_exponent: false,
            },
            labeling: LabelingConfig {
                method: LabelingMethod::TripleBarrier,
                barrier_width: 0.02,
                min_return: 0.005,
            },
            model_type: ModelType::RandomForest,
            training: TrainingConfig {
                test_size: 0.3,
                cv_folds: 5,
            },
        }
    }
}

impl ConfigSection for MLConfig {
    fn section_name() -> &'static str {
        "ml"
    }

    fn validate(&self) -> Result<(), TradebiasError> {
        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        todo!()
    }
}
```

#### 1.6 Create ConfigManager
**File**: `src/config/manager.rs` (NEW)

```rust
use super::{
    backtesting::BacktestingConfig,
    evolution::EvolutionConfig,
    ml::MLConfig,
    trade_management::TradeManagementConfig,
    traits::ConfigSection,
};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub evolution: EvolutionConfig,
    pub backtesting: BacktestingConfig,
    pub trade_management: TradeManagementConfig,
    pub ml: MLConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            evolution: EvolutionConfig::default(),
            backtesting: BacktestingConfig::default(),
            trade_management: TradeManagementConfig::default(),
            ml: MLConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), TradebiasError> {
        self.evolution.validate()?;
        self.backtesting.validate()?;
        self.trade_management.validate()?;
        self.ml.validate()?;
        Ok(())
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), TradebiasError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| TradebiasError::Configuration(format!("Failed to read config: {}", e)))?;

        let config: AppConfig = toml::from_str(&contents)
            .map_err(|e| TradebiasError::Configuration(format!("Failed to parse config: {}", e)))?;

        config.validate()?;

        *self.config.write().unwrap() = config;
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), TradebiasError> {
        let config = self.config.read().unwrap();
        let toml_str = toml::to_string_pretty(&*config)
            .map_err(|e| TradebiasError::Configuration(format!("Failed to serialize: {}", e)))?;

        std::fs::write(path, toml_str)
            .map_err(|e| TradebiasError::Configuration(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    pub fn update<F>(&self, f: F) -> Result<(), TradebiasError>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.write().unwrap();
        f(&mut config);
        config.validate()?;
        Ok(())
    }
}
```

#### 1.7 Update config/mod.rs
**File**: `src/config/mod.rs`

```rust
pub mod traits;
pub mod evolution;
pub mod backtesting;
pub mod trade_management;
pub mod ml;
pub mod manager;

pub use manager::{ConfigManager, AppConfig};
pub use evolution::EvolutionConfig;
pub use backtesting::BacktestingConfig;
pub use trade_management::TradeManagementConfig;
pub use ml::MLConfig;
```

#### 1.8 Add Configuration Error Variant
**File**: `src/error.rs`

```rust
// Add to TradebiasError enum:
Configuration(String),

// Add to Display impl:
TradebiasError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
```

#### 1.9 Add toml Dependency
**File**: `Cargo.toml`

```toml
[dependencies]
toml = "0.8"
```

### 2. Fix Polars DSL Issues in Primitives

**Status**: Partially Implemented (Polars DSL issues identified)
**Files**: `src/functions/primitives.rs`

#### 2.1 Verify Polars Methods
Review and fix all Polars method calls that may be using outdated API:

```rust
// If abs() is missing, use alternative:
.map(|s| s.abs(), GetOutput::default())  // Old API
// Or:
.abs()  // Should work in polars 0.36+

// Check documentation for current Polars version
```

### 3. Fix Polars DSL Issues in Tier 1 Indicators

**Status**: Partially Implemented (Polars DSL issues identified)
**Files**: `src/functions/momentum.rs`, `src/functions/volatility.rs`, `src/functions/volume.rs`

#### 3.1 Fix diff() method
**File**: `src/functions/momentum.rs`

```rust
// Old (if broken):
.diff(1, Default::default())

// New (check Polars version):
.diff(1)
// Or use alternative implementation
```

#### 3.2 Fix cum_sum() method
**File**: `src/functions/volume.rs`

```rust
// Old:
.cum_sum(false)

// New:
.cumsum(false)  // Note: cumsum not cum_sum
```

### 4. Complete Robustness Validation

**Status**: Partially Implemented (blocking compilation issues)
**Spec**: `docs/ai-implementation/16-robustness-validation.md`

#### 4.1 Fix Import Errors
After Phase 1 completion, verify all robustness validation files compile:

- `src/engines/validation/robustness/base.rs`
- `src/engines/validation/robustness/monte_carlo.rs`
- `src/engines/validation/robustness/parameter_stability.rs`
- `src/engines/validation/robustness/friction.rs`
- `src/engines/validation/orchestrator.rs`

Fix any remaining type mismatches or import errors.

### 5. Complete Code Generation & Semantic Generation

**Status**: Partially Implemented
**Files**:
- `src/engines/generation/semantic_mapper.rs`
- `src/engines/generation/ast.rs`

#### 5.1 Verify SemanticMapper Updates
After trait architecture fix in Phase 1, ensure semantic_mapper.rs compiles and properly:

- Uses FunctionRegistry correctly
- Builds StrategyAST or AstNode consistently
- Handles all function types properly

### 6. Add Missing Error Variant (if not in Phase 1)

**File**: `src/error.rs`

Ensure all required error variants exist:
```rust
pub enum TradebiasError {
    // Existing variants...
    Generation(String),
    Validation(String),
    Computation(String),
    Configuration(String),
}
```

## Implementation Order

1. **Configuration System** (Tasks 1.1-1.9) - Critical missing feature
2. **Polars DSL fixes** (Tasks 2.1, 3.1-3.2) - Enables indicators to work
3. **Robustness validation** (Task 4.1) - Complete existing partial work
4. **Semantic generation** (Task 5.1) - Verify after trait fixes
5. **Error variants** (Task 6) - If not already done in Phase 1

## Success Criteria

After Phase 2 completion:
- [ ] Configuration system fully implemented and functional
- [ ] All Polars DSL calls use correct API
- [ ] Robustness validation compiles without errors
- [ ] Semantic mapper works with new trait architecture
- [ ] `cargo build` still succeeds
- [ ] All "Partially Implemented" items now "Implemented"

## Notes

- Configuration system is highest priority - it's completely missing
- Polars fixes may require checking actual Polars version in use
- Test semantic_mapper thoroughly after trait refactor
- Consider adding integration tests after implementation (separate phase)
