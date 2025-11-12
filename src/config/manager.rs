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
