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

        // Trend Indicators
        metadata.insert(
            "SMA".to_string(),
            IndicatorMetadata {
                full_name: "Simple Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![10, 20, 50, 100, 200]),
            },
        );
        metadata.insert(
            "EMA".to_string(),
            IndicatorMetadata {
                full_name: "Exponential Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![10, 20, 50, 100, 200]),
            },
        );
        metadata.insert(
            "MACD".to_string(),
            IndicatorMetadata {
                full_name: "Moving Average Convergence Divergence".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![12, 26, 9]), // Fast, Slow, Signal
            },
        );
        metadata.insert(
            "BB".to_string(),
            IndicatorMetadata {
                full_name: "Bollinger Bands".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![20]), // Period
            },
        );
        metadata.insert(
            "Envelopes".to_string(),
            IndicatorMetadata {
                full_name: "Envelopes".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![14, 20]),
            },
        );
        metadata.insert(
            "SAR".to_string(),
            IndicatorMetadata {
                full_name: "Parabolic SAR".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: None, // Uses step and max, not periods
            },
        );
        metadata.insert(
            "Bears".to_string(),
            IndicatorMetadata {
                full_name: "Bears Power".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![13, 14]),
            },
        );
        metadata.insert(
            "Bulls".to_string(),
            IndicatorMetadata {
                full_name: "Bulls Power".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![13, 14]),
            },
        );
        metadata.insert(
            "DEMA".to_string(),
            IndicatorMetadata {
                full_name: "Double Exponential Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![9, 14, 21]),
            },
        );
        metadata.insert(
            "TEMA".to_string(),
            IndicatorMetadata {
                full_name: "Triple Exponential Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![9, 14, 21]),
            },
        );
        metadata.insert(
            "TriX".to_string(),
            IndicatorMetadata {
                full_name: "Triple Exponential Average".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "trend".to_string(),
                typical_periods: Some(vec![14, 15, 30]),
            },
        );

        // Momentum Indicators
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
            "Stochastic".to_string(),
            IndicatorMetadata {
                full_name: "Stochastic Oscillator".to_string(),
                scale: ScaleType::Oscillator0_100,
                value_range: Some((0.0, 100.0)),
                category: "momentum".to_string(),
                typical_periods: Some(vec![5, 3, 3]), // k, d, slowing
            },
        );
        metadata.insert(
            "CCI".to_string(),
            IndicatorMetadata {
                full_name: "Commodity Channel Index".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None, // Unbounded
                category: "momentum".to_string(),
                typical_periods: Some(vec![14, 20]),
            },
        );
        metadata.insert(
            "WilliamsR".to_string(),
            IndicatorMetadata {
                full_name: "Williams' %R".to_string(),
                scale: ScaleType::Oscillator0_100, // Technically -100 to 0
                value_range: Some((-100.0, 0.0)),
                category: "momentum".to_string(),
                typical_periods: Some(vec![14]),
            },
        );
        metadata.insert(
            "ROC".to_string(),
            IndicatorMetadata {
                full_name: "Rate of Change".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "momentum".to_string(),
                typical_periods: Some(vec![9, 12, 14]),
            },
        );
        metadata.insert(
            "AC".to_string(),
            IndicatorMetadata {
                full_name: "Accelerator Oscillator".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "momentum".to_string(),
                typical_periods: None, // Fixed periods (5, 34)
            },
        );
        metadata.insert(
            "AO".to_string(),
            IndicatorMetadata {
                full_name: "Awesome Oscillator".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "momentum".to_string(),
                typical_periods: None, // Fixed periods (5, 34)
            },
        );
        metadata.insert(
            "RVI".to_string(),
            IndicatorMetadata {
                full_name: "Relative Vigor Index".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "momentum".to_string(),
                typical_periods: Some(vec![10, 14]),
            },
        );
        metadata.insert(
            "DeMarker".to_string(),
            IndicatorMetadata {
                full_name: "DeMarker".to_string(),
                scale: ScaleType::Oscillator0_100, // Range is 0 to 1
                value_range: Some((0.0, 1.0)),
                category: "momentum".to_string(),
                typical_periods: Some(vec![13, 14]),
            },
        );
        metadata.insert(
            "Momentum".to_string(),
            IndicatorMetadata {
                full_name: "Momentum".to_string(),
                scale: ScaleType::OscillatorCentered,
                value_range: None,
                category: "momentum".to_string(),
                typical_periods: Some(vec![10, 12, 14]),
            },
        );

        // Volatility Indicators
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
        metadata.insert(
            "ADX".to_string(),
            IndicatorMetadata {
                full_name: "Average Directional Index".to_string(),
                scale: ScaleType::Oscillator0_100,
                value_range: Some((0.0, 100.0)),
                category: "volatility".to_string(),
                typical_periods: Some(vec![14]),
            },
        );
        metadata.insert(
            "StdDev".to_string(),
            IndicatorMetadata {
                full_name: "Standard Deviation".to_string(),
                scale: ScaleType::VolatilityDecimal,
                value_range: Some((0.0, f64::MAX)),
                category: "volatility".to_string(),
                typical_periods: Some(vec![20]),
            },
        );

        // Volume Indicators
        metadata.insert(
            "OBV".to_string(),
            IndicatorMetadata {
                full_name: "On-Balance Volume".to_string(),
                scale: ScaleType::Volume,
                value_range: None,
                category: "volume".to_string(),
                typical_periods: None,
            },
        );
        metadata.insert(
            "MFI".to_string(),
            IndicatorMetadata {
                full_name: "Money Flow Index".to_string(),
                scale: ScaleType::Oscillator0_100,
                value_range: Some((0.0, 100.0)),
                category: "volume".to_string(),
                typical_periods: Some(vec![14]),
            },
        );
        metadata.insert(
            "Force".to_string(),
            IndicatorMetadata {
                full_name: "Force Index".to_string(),
                scale: ScaleType::Volume,
                value_range: None,
                category: "volume".to_string(),
                typical_periods: Some(vec![1, 13]),
            },
        );
        metadata.insert(
            "Volumes".to_string(),
            IndicatorMetadata {
                full_name: "Volumes".to_string(),
                scale: ScaleType::Volume,
                value_range: None,
                category: "volume".to_string(),
                typical_periods: None,
            },
        );
        metadata.insert(
            "Chaikin".to_string(),
            IndicatorMetadata {
                full_name: "Chaikin Oscillator".to_string(),
                scale: ScaleType::Volume,
                value_range: None,
                category: "volume".to_string(),
                typical_periods: Some(vec![3, 10]), // Fast, Slow
            },
        );
        metadata.insert(
            "BWMFI".to_string(),
            IndicatorMetadata {
                full_name: "Market Facilitation Index".to_string(),
                scale: ScaleType::Volume,
                value_range: None,
                category: "volume".to_string(),
                typical_periods: None,
            },
        );

        Self { metadata }
    }

    pub fn get(&self, indicator: &str) -> Option<&IndicatorMetadata> {
        self.metadata.get(indicator)
    }

    pub fn get_all_keys(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
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
