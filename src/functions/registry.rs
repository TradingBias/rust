use crate::functions::{
    indicators::{
        momentum::{
            AC, AO, CCI,
            DeMarker, Momentum, RSI, RVI, Stochastic, WilliamsR,
        },
        trend::{
            Bears, BollingerBands, Bulls, DEMA, EMA, Envelopes, MACD, SAR, SMA, TEMA,
            TriX,
        },
        volatility::{ADX, ATR, StdDev},
        volume::{BWMFI, Chaikin, Force, MFI, OBV, Volumes},
    },
    primitives::{self, And, Or, Abs},
};
use std::{collections::HashMap, sync::Arc};

use super::{
    strategy::StrategyFunction,
    traits::{Indicator, Primitive},
};
use crate::types::DataType;

pub struct FunctionRegistry {
    functions: HashMap<String, StrategyFunction>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_indicators();
        registry.register_primitives();
        registry
    }

    pub fn get_function(&self, name: &str) -> Option<StrategyFunction> {
        self.functions.get(name).cloned()
    }

    pub fn get_indicator(&self, name: &str) -> Option<Arc<dyn Indicator>> {
        self.get_function(name)
            .and_then(|f| f.get_indicator_arc())
    }

    pub fn get_primitive(&self, name: &str) -> Option<Arc<dyn Primitive>> {
        self.get_function(name)
            .and_then(|f| f.get_primitive_arc())
    }

    pub fn get_by_output_type(&self, data_type: DataType) -> Vec<StrategyFunction> {
        self.functions
            .values()
            .filter(|f| f.output_type() == data_type)
            .cloned()
            .collect()
    }

    pub fn get_indicators(&self) -> Vec<Arc<dyn Indicator>> {
        self.functions
            .values()
            .filter_map(|f| f.get_indicator_arc())
            .collect()
    }

    fn register_indicators(&mut self) {
        let indicators: Vec<Arc<dyn Indicator>> = vec![
            Arc::new(RSI::new(14)),
            Arc::new(Stochastic::new(14, 3, 3)),
            Arc::new(CCI::new(14)),
            Arc::new(WilliamsR::new(14)),
            Arc::new(Momentum::new(14)),
            Arc::new(AC::new()),
            Arc::new(AO::new()),
            Arc::new(RVI::new(10)),
            Arc::new(DeMarker::new(14)),
            Arc::new(SMA::new(14)),
            Arc::new(EMA::new(14)),
            Arc::new(MACD::new(12, 26, 9)),
            Arc::new(BollingerBands::new(20, 2.0)),
            Arc::new(Envelopes::new(14, 0.1)),
            Arc::new(SAR::new(0.02, 0.2)),
            Arc::new(Bears::new(13)),
            Arc::new(Bulls::new(13)),
            Arc::new(DEMA::new(14)),
            Arc::new(TEMA::new(14)),
            Arc::new(TriX::new(14)),
            Arc::new(ATR::new(14)),
            Arc::new(ADX::new(14)),
            Arc::new(StdDev::new(14)),
            Arc::new(OBV::new()),
            Arc::new(MFI::new(14)),
            Arc::new(Force::new(13)),
            Arc::new(Volumes::new()),
            Arc::new(Chaikin::new(3, 10)),
            Arc::new(BWMFI::new()),
        ];

        for indicator in indicators {
            self.functions
                .insert(indicator.alias().to_string(), StrategyFunction::Indicator(indicator));
        }
    }

    fn register_primitives(&mut self) {
        let primitives: Vec<Arc<dyn Primitive>> = vec![Arc::new(And {}), Arc::new(Or {}), Arc::new(Abs {})];
        for primitive in primitives {
            self.functions
                .insert(primitive.alias().to_string(), StrategyFunction::Primitive(primitive));
        }
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_indicator_retrieval() {
        let registry = FunctionRegistry::new();
        let rsi_indicator = registry.get_indicator("RSI");
        assert!(rsi_indicator.is_some());
        assert_eq!(rsi_indicator.unwrap().alias(), "RSI");
    }

    #[test]
    fn test_registry_primitive_retrieval() {
        let registry = FunctionRegistry::new();
        let and_primitive = registry.get_primitive("And");
        assert!(and_primitive.is_some());
    }

    #[test]
    fn test_indicator_not_found() {
        let registry = FunctionRegistry::new();
        let non_existent = registry.get_indicator("NonExistent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_primitive_not_found() {
        let registry = FunctionRegistry::new();
        let non_existent = registry.get_primitive("NonExistent");
        assert!(non_existent.is_none());
    }
}
