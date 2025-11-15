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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StopLossConfig {
    FixedPercent { percent: f64 },
    ATR { multiplier: f64, period: usize },
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TakeProfitConfig {
    FixedPercent { percent: f64 },
    RiskReward { ratio: f64 },
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
