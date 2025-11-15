use polars::prelude::*;
use anyhow::Result;
use std::any::Any;
use crate::types::{DataType, ScaleType};

/// Calculation mode for indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationMode {
    /// Vectorized calculation using Polars (best performance when mathematically feasible)
    Vectorized,
    /// Stateful bar-by-bar calculation (for complex indicators where vectorization isn't practical)
    Stateful,
}

/// Base trait for all indicators
pub trait Indicator: Send + Sync + Any {
    /// Display name
    fn ui_name(&self) -> &'static str;

    /// Alias for use in strategy AST
    fn alias(&self) -> &'static str;

    /// Semantic scale type
    fn scale_type(&self) -> ScaleType;

    /// Expected value range
    fn value_range(&self) -> Option<(f64, f64)>;

    /// Number of parameters
    fn arity(&self) -> usize;

    /// Input types
    fn input_types(&self) -> Vec<DataType>;

    /// Output type
    fn output_type(&self) -> DataType;

    /// Returns the calculation mode for this indicator
    fn calculation_mode(&self) -> CalculationMode;

    /// Generate MQL5 code for this indicator (always stateful for live trading)
    fn generate_mql5(&self, args: &[String]) -> String;

    /// Call vectorized calculation if this indicator supports it
    /// Returns None if the indicator doesn't implement VectorizedIndicator
    fn try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<Expr>> {
        None
    }
}

/// Trait for vectorized indicators (used in backtesting)
pub trait VectorizedIndicator: Indicator {
    /// Calculate over entire series using Polars expressions
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr>;

    /// Provide try_calculate_vectorized implementation for Indicator trait
    fn provide_try_calculate_vectorized(&self, args: &[IndicatorArg]) -> Option<Result<Expr>> {
        Some(self.calculate_vectorized(args))
    }
}

/// Trait for stateful indicators (used when vectorization isn't practical)
pub trait StatefulIndicator: Indicator {
    /// Calculate single bar with state
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64>;

    /// Initialize state for stateful calculation
    fn init_state(&self) -> Box<dyn Any>;
}

/// Flexible argument for indicator calls
#[derive(Debug, Clone)]
pub enum IndicatorArg {
    Series(Expr),   // Polars expression
    Scalar(f64),    // Period, threshold, etc.
}

/// Macro to automatically implement try_calculate_vectorized for indicators
/// that implement VectorizedIndicator
#[macro_export]
macro_rules! impl_vectorized_indicator_bridge {
    ($type:ty) => {
        impl $type {
            /// Bridge method that connects Indicator::try_calculate_vectorized
            /// to VectorizedIndicator::calculate_vectorized
            #[inline]
            fn bridge_try_calculate_vectorized(&self, args: &[$crate::functions::traits::IndicatorArg]) -> Option<Result<polars::prelude::Expr>> {
                Some(self.calculate_vectorized(args))
            }
        }
    };
}

/// Primitive function trait
pub trait Primitive: Send + Sync {
    fn ui_name(&self) -> &'static str;
    fn alias(&self) -> &'static str;
    fn arity(&self) -> usize;
    fn input_types(&self) -> Vec<DataType>;
    fn output_type(&self) -> DataType;

    /// Execute primitive (always vectorized)
    fn execute(&self, args: &[Expr]) -> Result<Expr>;
    
    /// Generate MQL5 code
    fn generate_mql5(&self, args: &[String]) -> String;
}
