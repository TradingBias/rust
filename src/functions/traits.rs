use polars::prelude::*;
use anyhow::Result;
use std::any::Any;
use crate::types::{DataType, ScaleType};

/// Base trait for all indicators
pub trait Indicator: Send + Sync {
    /// Unique identifier
    fn alias(&self) -> &'static str;
    
    /// Display name
    fn ui_name(&self) -> &'static str;
    
    /// Semantic scale type
    fn scale_type(&self) -> ScaleType;
    
    /// Expected value range
    fn value_range(&self) -> Option<(f64, f64)>;
    
    /// Number of parameters
    fn arity(&self) -> usize;
    
    /// Input data types
    fn input_types(&self) -> Vec<DataType>;
    
    /// Output type
    fn output_type(&self) -> DataType {
        DataType::NumericSeries
    }
    
    /// VECTORIZED: Calculate over entire series (backtesting)
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr>;
    
    /// STATEFUL: Calculate single bar with state (live trading)
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64>;
    
    /// Initialize state for stateful calculation
    fn init_state(&self) -> Box<dyn Any>;
    
    /// Generate MQL5 code for this indicator
    fn generate_mql5(&self, args: &[String]) -> String;
}

/// Flexible argument for indicator calls
#[derive(Debug, Clone)]
pub enum IndicatorArg {
    Series(Expr),   // Polars expression
    Scalar(f64),    // Period, threshold, etc.
}

/// Primitive function trait
pub trait Primitive: Send + Sync {
    fn alias(&self) -> &'static str;
    fn ui_name(&self) -> &'static str;
    fn arity(&self) -> usize;
    fn input_types(&self) -> Vec<DataType>;
    fn output_type(&self) -> DataType;
    
    /// Execute primitive (always vectorized)
    fn execute(&self, args: &[Expr]) -> Result<Expr>;
    
    /// Generate MQL5 code
    fn generate_mql5(&self, args: &[String]) -> String;
}
