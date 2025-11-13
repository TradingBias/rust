# 17 - Configuration System & Settings Management

## Goal
Implement a centralized configuration system that manages all application settings with type safety, validation, defaults, and UI manifest generation. This provides a single source of truth for all configurable parameters.

## Prerequisites
- **01-architecture.md** - Project structure
- **02-type-system.md** - Core types

## What You'll Create
1. `ConfigManager` - Singleton configuration manager
2. `SettingsSchema` - Type-safe settings structures
3. `ManifestBuilder` - UI manifest generation
4. Configuration categories: Evolution, Backtesting, Trade Management, ML
5. Runtime validation and persistence

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│              Configuration System                     │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ ConfigManager (Singleton)                      │  │
│  │  • Load from file/environment                  │  │
│  │  • Validate all settings                       │  │
│  │  • Provide dot-notation access                 │  │
│  │  • Hot reload support                          │  │
│  └────────────────┬───────────────────────────────┘  │
│                   │                                   │
│    ┌──────────────┴──────────────┐                   │
│    │                               │                   │
│  ┌─▼────────────┐       ┌─────────▼──────────┐      │
│  │   Settings   │       │   UI Manifests     │      │
│  │   Schemas    │       │   (for Frontend)   │      │
│  └──────────────┘       └────────────────────┘      │
│                                                       │
│  Categories:                                          │
│  ├─ Evolution      (population, generations, etc.)   │
│  ├─ Backtesting    (validation method, splits)       │
│  ├─ TradeManagement (stop loss, position sizing)     │
│  ├─ ML             (model types, features)           │
│  ├─ Data           (connectors, cache)               │
│  └─ Performance    (workers, backend)                │
└──────────────────────────────────────────────────────┘
```

## Configuration Categories

### 1. Evolution Settings
- Population size
- Number of generations
- Mutation rate, crossover rate
- Selection method
- Fitness objectives

### 2. Backtesting Settings
- Validation method (Simple, WFO)
- In-sample / out-of-sample split
- Initial capital
- Commission and slippage

### 3. Trade Management
- Stop loss (fixed, ATR-based)
- Take profit (fixed, risk-reward)
- Position sizing
- Max positions

### 4. ML Settings
- Feature types
- Labeling configuration
- Model types
- Training parameters

## Implementation

### Step 1: Configuration Traits

Create `src/config/traits.rs`:

```rust
use crate::error::TradeBiasError;
use serde::{Deserialize, Serialize};

/// Trait for configuration sections
pub trait ConfigSection: Serialize + for<'de> Deserialize<'de> + Default + Clone {
    fn section_name() -> &'static str;

    fn validate(&self) -> Result<(), TradeBiasError>;

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
    pub label: String,
    pub field_type: FieldType,
    pub default: serde_json::Value,
    pub validation: FieldValidation,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldType {
    Integer,
    Float,
    Boolean,
    String,
    Select { options: Vec<String> },
    MultiSelect { options: Vec<String> },
    Range { min: f64, max: f64, step: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    pub required: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub pattern: Option<String>,
}
```

### Step 2: Evolution Settings

Create `src/config/evolution.rs`:

```rust
use super::traits::*;
use crate::error::TradeBiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSettings {
    pub population_size: usize,
    pub generations: usize,
    pub genome_length: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_rate: f64,
    pub tournament_size: usize,
    pub hall_of_fame_size: usize,
    pub selection_method: SelectionMethod,
    pub fitness_objectives: Vec<String>,
    pub fitness_weights: Vec<f64>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectionMethod {
    Tournament,
    Roulette,
    Rank,
}

impl Default for EvolutionSettings {
    fn default() -> Self {
        Self {
            population_size: 100,
            generations: 50,
            genome_length: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elitism_rate: 0.1,
            tournament_size: 3,
            hall_of_fame_size: 20,
            selection_method: SelectionMethod::Tournament,
            fitness_objectives: vec!["sharpe_ratio".to_string()],
            fitness_weights: vec![1.0],
            seed: None,
        }
    }
}

impl ConfigSection for EvolutionSettings {
    fn section_name() -> &'static str {
        "evolution"
    }

    fn validate(&self) -> Result<(), TradeBiasError> {
        if self.population_size == 0 {
            return Err(TradeBiasError::Configuration(
                "Population size must be greater than 0".to_string(),
            ));
        }

        if self.generations == 0 {
            return Err(TradeBiasError::Configuration(
                "Generations must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.mutation_rate) {
            return Err(TradeBiasError::Configuration(
                "Mutation rate must be between 0 and 1".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.crossover_rate) {
            return Err(TradeBiasError::Configuration(
                "Crossover rate must be between 0 and 1".to_string(),
            ));
        }

        if self.fitness_objectives.len() != self.fitness_weights.len() {
            return Err(TradeBiasError::Configuration(
                "Fitness objectives and weights must have same length".to_string(),
            ));
        }

        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        ConfigManifest {
            section: Self::section_name().to_string(),
            fields: vec![
                FieldManifest {
                    name: "population_size".to_string(),
                    label: "Population Size".to_string(),
                    field_type: FieldType::Integer,
                    default: serde_json::json!(100),
                    validation: FieldValidation {
                        required: true,
                        min: Some(10.0),
                        max: Some(1000.0),
                        pattern: None,
                    },
                    help_text: Some("Number of strategies per generation".to_string()),
                },
                FieldManifest {
                    name: "generations".to_string(),
                    label: "Generations".to_string(),
                    field_type: FieldType::Integer,
                    default: serde_json::json!(50),
                    validation: FieldValidation {
                        required: true,
                        min: Some(1.0),
                        max: Some(500.0),
                        pattern: None,
                    },
                    help_text: Some("Number of evolution cycles to run".to_string()),
                },
                FieldManifest {
                    name: "mutation_rate".to_string(),
                    label: "Mutation Rate".to_string(),
                    field_type: FieldType::Range {
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                    },
                    default: serde_json::json!(0.1),
                    validation: FieldValidation {
                        required: true,
                        min: Some(0.0),
                        max: Some(1.0),
                        pattern: None,
                    },
                    help_text: Some("Probability of gene mutation".to_string()),
                },
                FieldManifest {
                    name: "selection_method".to_string(),
                    label: "Selection Method".to_string(),
                    field_type: FieldType::Select {
                        options: vec![
                            "Tournament".to_string(),
                            "Roulette".to_string(),
                            "Rank".to_string(),
                        ],
                    },
                    default: serde_json::json!("Tournament"),
                    validation: FieldValidation {
                        required: true,
                        min: None,
                        max: None,
                        pattern: None,
                    },
                    help_text: Some("Method for selecting parents".to_string()),
                },
            ],
        }
    }
}
```

### Step 3: Trade Management Settings

Create `src/config/trade_management.rs`:

```rust
use super::traits::*;
use crate::error::TradeBiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeManagementSettings {
    pub stop_loss: StopLossConfig,
    pub take_profit: TakeProfitConfig,
    pub position_sizing: PositionSizingConfig,
    pub max_positions: usize,
    pub commission_pct: f64,
    pub slippage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossConfig {
    pub method: StopLossMethod,
    pub fixed_pct: Option<f64>,
    pub atr_multiplier: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StopLossMethod {
    Fixed,
    ATRBased,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitConfig {
    pub method: TakeProfitMethod,
    pub fixed_pct: Option<f64>,
    pub risk_reward_ratio: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TakeProfitMethod {
    Fixed,
    RiskReward,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingConfig {
    pub method: PositionSizingMethod,
    pub fixed_size: Option<f64>,
    pub risk_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionSizingMethod {
    Fixed,
    RiskPercent,
}

impl Default for TradeManagementSettings {
    fn default() -> Self {
        Self {
            stop_loss: StopLossConfig {
                method: StopLossMethod::Fixed,
                fixed_pct: Some(0.01),
                atr_multiplier: None,
            },
            take_profit: TakeProfitConfig {
                method: TakeProfitMethod::RiskReward,
                fixed_pct: None,
                risk_reward_ratio: Some(2.0),
            },
            position_sizing: PositionSizingConfig {
                method: PositionSizingMethod::RiskPercent,
                fixed_size: None,
                risk_pct: Some(0.01),
            },
            max_positions: 1,
            commission_pct: 0.001,
            slippage_pct: 0.0005,
        }
    }
}

impl ConfigSection for TradeManagementSettings {
    fn section_name() -> &'static str {
        "trade_management"
    }

    fn validate(&self) -> Result<(), TradeBiasError> {
        // Validate stop loss
        match self.stop_loss.method {
            StopLossMethod::Fixed => {
                if self.stop_loss.fixed_pct.is_none() {
                    return Err(TradeBiasError::Configuration(
                        "Fixed stop loss requires fixed_pct".to_string(),
                    ));
                }
            }
            StopLossMethod::ATRBased => {
                if self.stop_loss.atr_multiplier.is_none() {
                    return Err(TradeBiasError::Configuration(
                        "ATR-based stop loss requires atr_multiplier".to_string(),
                    ));
                }
            }
            StopLossMethod::None => {}
        }

        // Validate commission and slippage
        if self.commission_pct < 0.0 || self.commission_pct > 0.1 {
            return Err(TradeBiasError::Configuration(
                "Commission must be between 0% and 10%".to_string(),
            ));
        }

        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        ConfigManifest {
            section: Self::section_name().to_string(),
            fields: vec![
                FieldManifest {
                    name: "stop_loss_method".to_string(),
                    label: "Stop Loss Method".to_string(),
                    field_type: FieldType::Select {
                        options: vec!["Fixed".to_string(), "ATRBased".to_string(), "None".to_string()],
                    },
                    default: serde_json::json!("Fixed"),
                    validation: FieldValidation {
                        required: true,
                        min: None,
                        max: None,
                        pattern: None,
                    },
                    help_text: Some("Method for calculating stop loss".to_string()),
                },
                FieldManifest {
                    name: "commission_pct".to_string(),
                    label: "Commission (%)".to_string(),
                    field_type: FieldType::Float,
                    default: serde_json::json!(0.1),
                    validation: FieldValidation {
                        required: true,
                        min: Some(0.0),
                        max: Some(10.0),
                        pattern: None,
                    },
                    help_text: Some("Commission per trade as percentage".to_string()),
                },
            ],
        }
    }
}
```

### Step 4: Configuration Manager

Create `src/config/manager.rs`:

```rust
use super::{
    evolution::EvolutionSettings,
    trade_management::TradeManagementSettings,
    traits::ConfigSection,
};
use crate::error::TradeBiasError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub evolution: EvolutionSettings,
    pub trade_management: TradeManagementSettings,
    // Add other sections as needed
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            evolution: EvolutionSettings::default(),
            trade_management: TradeManagementSettings::default(),
        }
    }
}

/// Singleton configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigManager {
    /// Create new configuration manager with defaults
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
        }
    }

    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, TradeBiasError> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            TradeBiasError::Configuration(format!("Failed to read config file: {}", e))
        })?;

        let config: AppConfig = serde_json::from_str(&contents).map_err(|e| {
            TradeBiasError::Configuration(format!("Failed to parse config: {}", e))
        })?;

        // Validate all sections
        config.evolution.validate()?;
        config.trade_management.validate()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), TradeBiasError> {
        let config = self.config.read().unwrap();
        let json = serde_json::to_string_pretty(&*config).map_err(|e| {
            TradeBiasError::Configuration(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, json).map_err(|e| {
            TradeBiasError::Configuration(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Get evolution settings
    pub fn evolution(&self) -> EvolutionSettings {
        self.config.read().unwrap().evolution.clone()
    }

    /// Update evolution settings
    pub fn set_evolution(&self, settings: EvolutionSettings) -> Result<(), TradeBiasError> {
        settings.validate()?;
        self.config.write().unwrap().evolution = settings;
        Ok(())
    }

    /// Get trade management settings
    pub fn trade_management(&self) -> TradeManagementSettings {
        self.config.read().unwrap().trade_management.clone()
    }

    /// Update trade management settings
    pub fn set_trade_management(
        &self,
        settings: TradeManagementSettings,
    ) -> Result<(), TradeBiasError> {
        settings.validate()?;
        self.config.write().unwrap().trade_management = settings;
        Ok(())
    }

    /// Get unified manifest for UI generation
    pub fn get_unified_manifest(&self) -> Vec<serde_json::Value> {
        let config = self.config.read().unwrap();
        vec![
            serde_json::to_value(config.evolution.to_manifest()).unwrap(),
            serde_json::to_value(config.trade_management.to_manifest()).unwrap(),
        ]
    }

    /// Get value by dot-notation path
    pub fn get_value(&self, path: &str) -> Option<serde_json::Value> {
        let config = self.config.read().unwrap();
        let json = serde_json::to_value(&*config).ok()?;

        // Navigate path
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &json;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current.clone())
    }
}

// Global singleton instance
static INSTANCE: std::sync::OnceLock<ConfigManager> = std::sync::OnceLock::new();

impl ConfigManager {
    /// Get global singleton instance
    pub fn global() -> &'static ConfigManager {
        INSTANCE.get_or_init(|| ConfigManager::new())
    }
}
```

### Step 5: Manifest Builder

Create `src/config/manifest_builder.rs`:

```rust
use super::traits::ConfigManifest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedManifest {
    pub version: String,
    pub sections: Vec<ConfigManifest>,
}

pub struct ManifestBuilder {
    sections: Vec<ConfigManifest>,
}

impl ManifestBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn add_section(mut self, section: ConfigManifest) -> Self {
        self.sections.push(section);
        self
    }

    pub fn build(self) -> UnifiedManifest {
        UnifiedManifest {
            version: env!("CARGO_PKG_VERSION").to_string(),
            sections: self.sections,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let manifest = UnifiedManifest {
            version: env!("CARGO_PKG_VERSION").to_string(),
            sections: self.sections.clone(),
        };
        serde_json::to_string_pretty(&manifest)
    }
}
```

## Usage Example

```rust
use tradebias::config::{ConfigManager, evolution::EvolutionSettings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from file
    let config_manager = ConfigManager::load_from_file("config.json")?;

    // Or use global singleton
    let config = ConfigManager::global();

    // Get settings
    let evolution_settings = config.evolution();
    println!("Population size: {}", evolution_settings.population_size);

    // Update settings
    let mut new_settings = evolution_settings.clone();
    new_settings.population_size = 200;
    config.set_evolution(new_settings)?;

    // Save to file
    config.save_to_file("config.json")?;

    // Get unified manifest for UI
    let manifests = config.get_unified_manifest();
    let json = serde_json::to_string_pretty(&manifests)?;
    println!("UI Manifest:\n{}", json);

    // Dot-notation access
    if let Some(value) = config.get_value("evolution.population_size") {
        println!("Population size via dot notation: {}", value);
    }

    Ok(())
}
```

## Configuration File Format

`config.json`:
```json
{
  "evolution": {
    "population_size": 100,
    "generations": 50,
    "mutation_rate": 0.1,
    "crossover_rate": 0.7,
    "elitism_rate": 0.1,
    "tournament_size": 3,
    "hall_of_fame_size": 20,
    "selection_method": "Tournament",
    "fitness_objectives": ["sharpe_ratio", "max_drawdown_pct"],
    "fitness_weights": [0.7, -0.3],
    "seed": null
  },
  "trade_management": {
    "stop_loss": {
      "method": "Fixed",
      "fixed_pct": 0.01,
      "atr_multiplier": null
    },
    "take_profit": {
      "method": "RiskReward",
      "fixed_pct": null,
      "risk_reward_ratio": 2.0
    },
    "position_sizing": {
      "method": "RiskPercent",
      "fixed_size": null,
      "risk_pct": 0.01
    },
    "max_positions": 1,
    "commission_pct": 0.001,
    "slippage_pct": 0.0005
  }
}
```

## Verification

### Test 1: Validation
```rust
#[test]
fn test_config_validation() {
    let mut settings = EvolutionSettings::default();

    // Valid
    assert!(settings.validate().is_ok());

    // Invalid: mutation rate out of range
    settings.mutation_rate = 1.5;
    assert!(settings.validate().is_err());
}
```

### Test 2: Persistence
```rust
#[test]
fn test_save_load() {
    let config = ConfigManager::new();

    // Save
    config.save_to_file("test_config.json").unwrap();

    // Load
    let loaded = ConfigManager::load_from_file("test_config.json").unwrap();

    assert_eq!(config.evolution().population_size, loaded.evolution().population_size);
}
```

## Next Steps

Proceed to **[18-data-connectors.md](./18-data-connectors.md)** to implement the data loading and caching system.
