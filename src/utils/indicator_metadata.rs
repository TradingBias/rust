use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetadata {
    pub full_name: String,
    pub scale: ScaleType,
    pub value_range: Option<(f64, f64)>,
    pub category: String,
    pub typical_periods: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScaleType {
    Price,              // Follows price (SMA, EMA, Bollinger)
    Oscillator0_100,    // 0-100 range (RSI, Stochastic)
    OscillatorCentered, // Zero-centered (MACD, Momentum)
    VolatilityDecimal,  // Small decimals (ATR, StdDev)
    Volume,             // Large integers (OBV, Volume)
    Ratio,              // Ratio-based (Williams %R)
    Index,              // Index-based (ADX, CCI)
}

pub struct MetadataRegistry {
    metadata: HashMap<String, IndicatorMetadata>,
}

impl MetadataRegistry {
    pub fn new() -> Self {
        let mut metadata = HashMap::new();

        // Trend indicators
        metadata.insert(
            "SMA".to_string(),
            IndicatorMetadata {
                full_name: "Simple Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None, // Follows price
                category: "trend".to_string(),
                typical_periods: Some(vec![5, 10, 14, 20, 50, 100, 200]),
            },
        );

        metadata.insert(
            "RSI".to_string(),
            IndicatorMetadata {
                full_name: "Relative Strength Index".to_string(),
                scale: ScaleType::Oscillator0_100,
                value_range: Some((0.0, 100.0)),
                category: "momentum".to_string(),
                typical_periods: Some(vec![9, 14, 21, 25]),
            },
        );

        metadata.insert(
            "ATR".to_string(),
            IndicatorMetadata {
                full_name: "Average True Range".to_string(),
                scale: ScaleType::VolatilityDecimal,
                value_range: Some((0.0, f64::MAX)),
                category: "volatility".to_string(),
                typical_periods: Some(vec![7, 14, 21]),
            },
        );

        // Add more indicators...

        Self { metadata }
    }

    pub fn get(&self, indicator: &str) -> Option<&IndicatorMetadata> {
        self.metadata.get(indicator)
    }

    /// Check if two indicators can be meaningfully compared
    pub fn are_compatible(&self, ind1: &str, ind2: &str) -> bool {
        match (self.get(ind1), self.get(ind2)) {
            (Some(meta1), Some(meta2)) => meta1.scale == meta2.scale,
            _ => false,
        }
    }

    /// Generate appropriate threshold for indicator
    pub fn generate_threshold(&self, indicator: &str, gene: u32) -> f64 {
        if let Some(meta) = self.get(indicator) {
            match meta.scale {
                ScaleType::Oscillator0_100 => {
                    // Common thresholds: 30, 70 (oversold/overbought)
                    let thresholds = [20.0, 30.0, 40.0, 60.0, 70.0, 80.0];
                    thresholds[(gene as usize) % thresholds.len()]
                }
                ScaleType::OscillatorCentered => {
                    // Zero-crossing or small thresholds
                    let thresholds = [-10.0, -5.0, 0.0, 5.0, 10.0];
                    thresholds[(gene as usize) % thresholds.len()]
                }
                ScaleType::VolatilityDecimal => {
                    // Small positive values
                    0.0001 + (gene as f64 / u32::MAX as f64) * 0.01
                }
                _ => (gene as f64 / u32::MAX as f64) * 100.0,
            }
        } else {
            (gene as f64 / u32::MAX as f64) * 100.0
        }
    }
}
