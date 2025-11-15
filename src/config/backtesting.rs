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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationMethod {
    Simple,
    WalkForwardAnchored,
    WalkForwardRolling,
    KFold,
}

impl Default for BacktestingConfig {
    fn default() -> Self {
        Self {
            validation_method: ValidationMethod::WalkForwardRolling,
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
