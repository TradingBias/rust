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
