use crate::types::DataType;
use std::any::Any;

pub trait StrategyFunction: Send + Sync + 'static + Any {
    fn alias(&self) -> &'static str;
    fn input_types(&self) -> Vec<DataType>;
    fn output_type(&self) -> DataType;
}
