use crate::types::DataType;
use crate::functions::traits::{Indicator, Primitive};
use std::sync::Arc;

#[derive(Clone)]
pub enum StrategyFunction {
    Indicator(Arc<dyn Indicator>),
    Primitive(Arc<dyn Primitive>),
}

impl StrategyFunction {
    pub fn name(&self) -> &'static str {
        match self {
            StrategyFunction::Indicator(i) => i.alias(),
            StrategyFunction::Primitive(p) => p.alias(),
        }
    }

    pub fn output_type(&self) -> DataType {
        match self {
            StrategyFunction::Indicator(i) => i.output_type(),
            StrategyFunction::Primitive(p) => p.output_type(),
        }
    }

    pub fn input_types(&self) -> Vec<DataType> {
        match self {
            StrategyFunction::Indicator(i) => i.input_types(),
            StrategyFunction::Primitive(p) => p.input_types(),
        }
    }

    pub fn as_indicator(&self) -> Option<&dyn Indicator> {
        match self {
            StrategyFunction::Indicator(i) => Some(i.as_ref()),
            _ => None,
        }
    }

    pub fn as_primitive(&self) -> Option<&dyn Primitive> {
        match self {
            StrategyFunction::Primitive(p) => Some(p.as_ref()),
            _ => None,
        }
    }
}
