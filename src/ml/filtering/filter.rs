use crate::ml::models::meta_model::MetaModel;
use crate::ml::signals::types::*;
use crate::error::TradebiasError;
use polars::prelude::*;

pub struct SignalFilter {
    model: MetaModel,
    threshold: f64,
}

impl SignalFilter {
    pub fn new(model: MetaModel, threshold: f64) -> Self {
        Self { model, threshold }
    }

    /// Filter signals based on ML predictions
    pub fn filter(
        &self,
        signals: &[Signal],
        features: &DataFrame,
    ) -> Result<Vec<Signal>, TradebiasError> {
        // Get predictions
        let probabilities = self.model.predict_proba(features)?;

        // Filter signals above threshold
        let mut filtered_signals = Vec::new();

        for (signal, &prob) in signals.iter().zip(probabilities.iter()) {
            if prob >= self.threshold {
                filtered_signals.push(signal.clone());
            }
        }

        Ok(filtered_signals)
    }

    /// Analyze filtering impact
    pub fn analyze_impact(
        &self,
        original_signals: &[Signal],
        filtered_signals: &[Signal],
    ) -> FilteringStats {
        FilteringStats {
            original_count: original_signals.len(),
            filtered_count: filtered_signals.len(),
            retention_rate: (filtered_signals.len() as f64 / original_signals.len() as f64) * 100.0,
            rejected_count: original_signals.len() - filtered_signals.len(),
        }
    }
}

#[derive(Debug)]
pub struct FilteringStats {
    pub original_count: usize,
    pub filtered_count: usize,
    pub retention_rate: f64,
    pub rejected_count: usize,
}
