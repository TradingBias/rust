use std::collections::HashMap;
use std::sync::Arc;
use crate::functions::manifest::{IndicatorManifest, TIER1_INDICATORS, TIER2_INDICATORS};
use crate::functions::primitives;
use crate::functions::traits::{Indicator, Primitive};
use crate::functions::indicators::{
    trend::{SMA, EMA, MACD, BollingerBands},
    momentum::{RSI},
};

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
            "SMA" => Some(Arc::new(SMA { period: 14 })), // Default period
            "EMA" => Some(Arc::new(EMA { period: 14 })),
            "RSI" => Some(Arc::new(RSI { period: 14 })),
            "MACD" => Some(Arc::new(MACD { fast_period: 12, slow_period: 26, signal_period: 9 })),
            "BB" => Some(Arc::new(BollingerBands { period: 20, deviation: 2.0 })),
            // Add other indicators here as they are implemented
            _ => None, // Indicator not yet implemented
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
