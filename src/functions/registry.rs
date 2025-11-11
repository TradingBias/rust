use std::collections::HashMap;
use std::sync::Arc;
use crate::functions::manifest::{IndicatorManifest, TIER1_INDICATORS, TIER2_INDICATORS};
use crate::functions::primitives;
use crate::functions::traits::{Indicator, Primitive};
use crate::functions::indicators::*;

pub struct FunctionRegistry {
    primitives: HashMap<String, Arc<dyn Primitive>>,
    indicators: HashMap<String, Arc<dyn Indicator>>,
    manifest: IndicatorManifest,
    config: RegistryConfig,
}

pub struct RegistryConfig {
    pub primitive_weight: f64,
    pub enable_cache: bool,
    pub cache_size_mb: usize,
}

impl FunctionRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            indicators: HashMap::new(),
            manifest: IndicatorManifest::default(),
            config,
        };
        
        registry.register_primitives();
        registry.register_indicators();
        registry
    }
    
    fn register_primitives(&mut self) {
        self.add_primitive(primitives::MovingAverage { method: primitives::MAMethod::Simple });
        self.add_primitive(primitives::MovingAverage { method: primitives::MAMethod::Exponential });
        self.add_primitive(primitives::MovingAverage { method: primitives::MAMethod::Linear });
        self.add_primitive(primitives::MovingAverage { method: primitives::MAMethod::Smoothed });
        self.add_primitive(primitives::Highest);
        self.add_primitive(primitives::Lowest);
        self.add_primitive(primitives::Sum);
        self.add_primitive(primitives::StdDev);
        self.add_primitive(primitives::Momentum);
        self.add_primitive(primitives::Shift);
        self.add_primitive(primitives::Absolute);
        self.add_primitive(primitives::Divide);
        self.add_primitive(primitives::Multiply);
        self.add_primitive(primitives::Add);
        self.add_primitive(primitives::Subtract);
    }

    fn add_primitive<P: Primitive + 'static>(&mut self, primitive: P) {
        self.primitives.insert(primitive.alias().to_string(), Arc::new(primitive));
    }

    fn register_indicators(&mut self) {
        for &alias in TIER1_INDICATORS.iter().chain(TIER2_INDICATORS.iter()) {
            if let Some(indicator) = self.build_indicator(alias) {
                self.indicators.insert(alias.to_string(), indicator);
            }
        }
    }

    fn build_indicator(&self, alias: &str) -> Option<Arc<dyn Indicator>> {
        match alias {
            // Tier 1
            "SMA" => Some(Arc::new(SMA { period: 14 })),
            "EMA" => Some(Arc::new(EMA { period: 14 })),
            "RSI" => Some(Arc::new(RSI { period: 14 })),
            "MACD" => Some(Arc::new(MACD { fast_period: 12, slow_period: 26, signal_period: 9 })),
            "BB" => Some(Arc::new(BollingerBands { period: 20, deviation: 2.0 })),
            "ATR" => Some(Arc::new(ATR { period: 14 })),
            "Stochastic" => Some(Arc::new(Stochastic { k_period: 14, d_period: 3, slowing: 3 })),
            "ADX" => Some(Arc::new(ADX { period: 14 })),
            "OBV" => Some(Arc::new(OBV)),
            "CCI" => Some(Arc::new(CCI { period: 14 })),

            // Tier 2
            "WilliamsR" => Some(Arc::new(WilliamsR { period: 14 })),
            "MFI" => Some(Arc::new(MFI { period: 14 })),
            "ROC" => Some(Arc::new(ROC { period: 14 })),
            "DeMarker" => Some(Arc::new(DeMarker { period: 14 })),
            "StdDev" => Some(Arc::new(StdDev { period: 20 })),
            "Envelopes" => Some(Arc::new(Envelopes { period: 20, deviation: 0.1 })),
            "SAR" => Some(Arc::new(SAR { step: 0.02, max: 0.2 })),
            "Force" => Some(Arc::new(Force { period: 13 })),
            "Bears" => Some(Arc::new(Bears)),
            "Bulls" => Some(Arc::new(Bulls)),
            "Momentum" => Some(Arc::new(Momentum { period: 14 })),
            "DEMA" => Some(Arc::new(DEMA { period: 14 })),
            "TEMA" => Some(Arc::new(TEMA { period: 14 })),
            "RVI" => Some(Arc::new(RVI { period: 10 })),
            "TriX" => Some(Arc::new(TriX { period: 15 })),
            "Volumes" => Some(Arc::new(Volumes)),
            "Chaikin" => Some(Arc::new(Chaikin { fast_period: 3, slow_period: 10 })),
            "BWMFI" => Some(Arc::new(BWMFI)),
            "AC" => Some(Arc::new(AC)),
            "AO" => Some(Arc::new(AO)),

            _ => None,
        }
    }
    
    pub fn get_indicator(&self, alias: &str) -> Option<Arc<dyn Indicator>> {
        self.indicators.get(alias).cloned()
    }
    
    pub fn get_primitive(&self, alias: &str) -> Option<Arc<dyn Primitive>> {
        self.primitives.get(alias).cloned()
    }
    
    pub fn get_function(&self, alias: &str) -> Option<FunctionRef> {
        if let Some(prim) = self.get_primitive(alias) {
            return Some(FunctionRef::Primitive(prim));
        }
        if let Some(ind) = self.get_indicator(alias) {
            return Some(FunctionRef::Indicator(ind));
        }
        None
    }
}

pub enum FunctionRef {
    Primitive(Arc<dyn Primitive>),
    Indicator(Arc<dyn Indicator>),
}
