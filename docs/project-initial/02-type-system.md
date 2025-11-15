# 02 - Type System & Traits

## Goal
Implement the complete type system including core types, error handling, and trait definitions that all other modules will depend on.

## Prerequisites
- **[01-architecture.md](./01-architecture.md)** completed
- Project structure created
- `cargo check` runs successfully

## What You'll Create
1. Error types in `src/error.rs`
2. Core data types in `src/types.rs`
3. Indicator and primitive traits in `src/functions/traits.rs`

## Implementation Steps

### Step 1: Implement Error Types

Create `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradeBiasError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Code generation error: {0}")]
    CodeGenError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Indicator not found: {0}")]
    IndicatorNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::error::PolarsError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TradeBiasError>;
```

### Step 2: Implement Core Types

Create `src/types.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value scale information for indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    /// Follows price movements (e.g., SMA, EMA, Bollinger Bands)
    Price,
    /// Bounded oscillator 0-100 (e.g., RSI, Stochastic)
    Oscillator0_100,
    /// Zero-centered oscillator (e.g., MACD, Momentum)
    OscillatorCentered,
    /// Volatility measure (e.g., ATR, StdDev)
    Volatility,
    /// Volume-based (e.g., OBV)
    Volume,
    /// Ratio indicator (e.g., Williams %R)
    Ratio,
}

/// Data type for expressions and values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Polars Series of f64
    NumericSeries,
    /// Polars Series of bool
    BoolSeries,
    /// Scalar integer
    Integer,
    /// Scalar float
    Float,
}

/// Abstract Syntax Tree node for strategy expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    /// Constant value
    Const(Value),
    /// Function call with arguments
    Call {
        function: String,
        args: Vec<Box<AstNode>>,
    },
    /// Conditional rule (if condition then action)
    Rule {
        condition: Box<AstNode>,
        action: Box<AstNode>,
    },
}

/// Value types for constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// Trade record from backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_bar: usize,
    pub exit_bar: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub direction: Direction,
    pub size: f64,
    pub profit: f64,
    pub exit_reason: ExitReason,
    pub fees: f64,
}

/// Trade direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

/// Reason for trade exit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    Signal,
    EndOfData,
}

/// Complete strategy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub ast: AstNode,
    pub metrics: HashMap<String, f64>,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub in_sample: bool,
}

/// Moving average methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MAMethod {
    Simple,      // Simple Moving Average (arithmetic mean)
    Exponential, // Exponential Moving Average (with smoothing)
    Linear,      // Linear Weighted Moving Average
    Smoothed,    // Smoothed Moving Average (SMMA)
}
```

### Step 3: Implement Indicator Trait System

Create `src/functions/traits.rs`:

```rust
use crate::types::{DataType, ScaleType};
use crate::error::Result;
use polars::prelude::*;
use std::any::Any;

/// Calculation mode for indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationMode {
    /// Vectorized calculation using Polars (best performance)
    /// Use when indicator math can be expressed as Polars operations
    Vectorized,

    /// Stateful bar-by-bar calculation
    /// Use when vectorization isn't mathematically practical
    Stateful,
}

/// Base trait for all indicators
pub trait Indicator: Send + Sync {
    /// Unique identifier (e.g., "RSI", "MACD")
    fn alias(&self) -> &'static str;

    /// Display name for UI (e.g., "Relative Strength Index")
    fn ui_name(&self) -> &'static str;

    /// Semantic scale type (Price, Oscillator, etc.)
    fn scale_type(&self) -> ScaleType;

    /// Expected value range (e.g., Some((0.0, 100.0)) for RSI)
    fn value_range(&self) -> Option<(f64, f64)>;

    /// Number of parameters
    fn arity(&self) -> usize;

    /// Input data types (e.g., [NumericSeries, Integer])
    fn input_types(&self) -> Vec<DataType>;

    /// Output type (usually NumericSeries)
    fn output_type(&self) -> DataType {
        DataType::NumericSeries
    }

    /// Returns the calculation mode for this indicator
    fn calculation_mode(&self) -> CalculationMode;

    /// Generate MQL5 code for this indicator (always stateful for live trading)
    /// Args are MQL5 variable names like ["Close", "14"]
    fn generate_mql5(&self, args: &[String]) -> String;
}

/// Trait for vectorized indicators (preferred for backtesting)
pub trait VectorizedIndicator: Indicator {
    /// Calculate over entire series using Polars expressions
    /// Returns a Polars Expr that computes the indicator
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr>;
}

/// Trait for stateful indicators (use when vectorization isn't practical)
pub trait StatefulIndicator: Indicator {
    /// Calculate single bar with state
    /// Args are the current bar values (e.g., close price, high, low)
    /// State is mutable and persists between bars
    fn calculate_stateful(&self, args: &[f64], state: &mut dyn Any) -> Result<f64>;

    /// Initialize state for stateful calculation
    /// Returns a boxed state object that will be passed to calculate_stateful
    fn init_state(&self) -> Box<dyn Any>;
}

/// Flexible argument for indicator calls
#[derive(Debug, Clone)]
pub enum IndicatorArg {
    /// Polars expression (e.g., col("close"))
    Series(Expr),
    /// Scalar parameter (e.g., period = 14)
    Scalar(f64),
}

impl IndicatorArg {
    /// Extract scalar value or return error
    pub fn as_scalar(&self) -> Result<f64> {
        match self {
            IndicatorArg::Scalar(v) => Ok(*v),
            IndicatorArg::Series(_) => Err(crate::error::TradeBiasError::TypeMismatch {
                expected: "Scalar".to_string(),
                actual: "Series".to_string(),
            }),
        }
    }

    /// Extract series expression or return error
    pub fn as_series(&self) -> Result<Expr> {
        match self {
            IndicatorArg::Series(e) => Ok(e.clone()),
            IndicatorArg::Scalar(_) => Err(crate::error::TradeBiasError::TypeMismatch {
                expected: "Series".to_string(),
                actual: "Scalar".to_string(),
            }),
        }
    }
}

/// Primitive function trait (building blocks for indicators)
pub trait Primitive: Send + Sync {
    /// Unique identifier (e.g., "SMA", "Highest")
    fn alias(&self) -> &'static str;

    /// Display name for UI
    fn ui_name(&self) -> &'static str;

    /// Number of parameters
    fn arity(&self) -> usize;

    /// Input data types
    fn input_types(&self) -> Vec<DataType>;

    /// Output type
    fn output_type(&self) -> DataType;

    /// Execute primitive (always vectorized)
    /// Args are Polars expressions
    fn execute(&self, args: &[Expr]) -> Result<Expr>;

    /// Generate MQL5 code for this primitive
    /// Args are MQL5 variable names
    fn generate_mql5(&self, args: &[String]) -> String;
}
```

### Step 4: Update src/lib.rs

Update `src/lib.rs` to export the new types:

```rust
// Public API
pub mod config;
pub mod data;
pub mod functions;
pub mod engines;
pub mod ml;
pub mod codegen;
pub mod utils;

// Core types
pub mod error;
pub mod types;

// Re-export commonly used types
pub use error::{TradeBiasError, Result};
pub use types::*;
pub use functions::traits::*;

// Public API functions (implementations come later)
pub fn run_evolution() -> Result<()> {
    todo!("To be implemented in engines module")
}

pub fn run_backtest() -> Result<()> {
    todo!("To be implemented in engines/evaluation")
}

pub fn train_ml_model() -> Result<()> {
    todo!("To be implemented in ml module")
}

pub fn generate_mql5_ea() -> Result<String> {
    todo!("To be implemented in codegen module")
}
```

### Step 5: Update src/functions/mod.rs

Update `src/functions/mod.rs`:

```rust
pub mod traits;
pub mod primitives;
pub mod indicators;
pub mod manifest;
pub mod registry;

// Re-export trait system
pub use traits::*;
```

### Step 6: Create Placeholder Files

Create these placeholder files to avoid compilation errors:

**src/functions/primitives.rs**:
```rust
use crate::functions::traits::*;
use crate::error::Result;
use polars::prelude::*;

// Primitives will be implemented in Phase 2
// This file is a placeholder for now
```

**src/functions/manifest.rs**:
```rust
use crate::types::ScaleType;

pub struct IndicatorManifest {
    pub tier1: Vec<ComposedIndicator>,
    pub tier2: Vec<ComposedIndicator>,
    pub tier3: Vec<ComposedIndicator>,
}

pub struct ComposedIndicator {
    pub alias: String,
    pub ui_name: String,
    pub scale_type: ScaleType,
    pub value_range: Option<(f64, f64)>,
}

impl IndicatorManifest {
    pub fn default() -> Self {
        Self {
            tier1: Vec::new(),
            tier2: Vec::new(),
            tier3: Vec::new(),
        }
    }
}
```

**src/functions/registry.rs**:
```rust
use crate::functions::traits::*;
use crate::functions::manifest::IndicatorManifest;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FunctionRegistry {
    primitives: HashMap<String, Arc<dyn Primitive>>,
    indicators: HashMap<String, Arc<dyn Indicator>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            primitives: HashMap::new(),
            indicators: HashMap::new(),
        }
    }
}
```

## Verification

After completing this phase, verify:

### 1. Project Compiles

```bash
cargo check
```

Should succeed with no errors (warnings about unused code are OK).

### 2. Type System is Complete

Check that all these types compile:

```rust
// Test in a new file: tests/type_system_test.rs
use tradebias::*;

#[test]
fn test_scale_type() {
    let scale = ScaleType::Price;
    assert_eq!(scale, ScaleType::Price);
}

#[test]
fn test_data_type() {
    let dt = DataType::NumericSeries;
    assert_eq!(dt, DataType::NumericSeries);
}

#[test]
fn test_trade() {
    let trade = Trade {
        entry_bar: 0,
        exit_bar: 10,
        entry_price: 100.0,
        exit_price: 105.0,
        direction: Direction::Long,
        size: 1.0,
        profit: 5.0,
        exit_reason: ExitReason::TakeProfit,
        fees: 0.1,
    };
    assert_eq!(trade.profit, 5.0);
}

#[test]
fn test_indicator_arg() {
    use polars::prelude::*;
    let arg = IndicatorArg::Scalar(14.0);
    assert_eq!(arg.as_scalar().unwrap(), 14.0);
    assert!(arg.as_series().is_err());
}
```

Run the test:

```bash
cargo test type_system_test
```

### 3. Error Types Work

Test error handling:

```rust
// Add to tests/type_system_test.rs
#[test]
fn test_error_types() {
    use tradebias::TradeBiasError;

    let err = TradeBiasError::IndicatorNotFound("RSI".to_string());
    assert!(err.to_string().contains("RSI"));

    let err = TradeBiasError::TypeMismatch {
        expected: "Series".to_string(),
        actual: "Scalar".to_string(),
    };
    assert!(err.to_string().contains("Series"));
    assert!(err.to_string().contains("Scalar"));
}
```

### 4. Trait System Compiles

Check that trait definitions are correct by trying to implement a dummy indicator:

```rust
// Add to tests/type_system_test.rs
use tradebias::*;
use polars::prelude::*;

struct DummyIndicator;

impl Indicator for DummyIndicator {
    fn alias(&self) -> &'static str { "DUMMY" }
    fn ui_name(&self) -> &'static str { "Dummy Indicator" }
    fn scale_type(&self) -> ScaleType { ScaleType::Price }
    fn value_range(&self) -> Option<(f64, f64)> { None }
    fn arity(&self) -> usize { 1 }
    fn input_types(&self) -> Vec<DataType> { vec![DataType::NumericSeries] }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }
    fn generate_mql5(&self, _args: &[String]) -> String { "".to_string() }
}

impl VectorizedIndicator for DummyIndicator {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        // Dummy implementation
        Ok(col("close"))
    }
}

#[test]
fn test_dummy_indicator() {
    let ind = DummyIndicator;
    assert_eq!(ind.alias(), "DUMMY");
    assert_eq!(ind.calculation_mode(), CalculationMode::Vectorized);
}
```

Run:

```bash
cargo test test_dummy_indicator
```

## Common Issues

### Issue: Trait object safety errors
**Solution**: Make sure all trait methods either:
- Return `Self` only in `Sized` contexts
- Use `Box<dyn Any>` for state instead of generic types
- Use `&'static str` instead of `String` for constant identifiers

### Issue: Polars import errors
**Solution**: Make sure `Cargo.toml` has:
```toml
polars = { version = "0.43", features = ["lazy", "temporal", "dtype-full"] }
```

### Issue: Circular dependency with traits
**Solution**: Keep all traits in `src/functions/traits.rs` and import them elsewhere. Never define traits in modules that implement them.

## Key Concepts

### Calculation Modes

**Vectorized Mode (Preferred)**:
- Use when indicator can be expressed as Polars operations
- 100-1000x faster for backtesting
- Examples: SMA, EMA, RSI, MACD, ATR, Bollinger Bands
- Implement `VectorizedIndicator` trait

**Stateful Mode (When Needed)**:
- Use when vectorization isn't mathematically practical
- Bar-by-bar calculation with state buffer
- Examples: ADX (complex multi-step with smoothing), SAR (complex conditional logic)
- Implement `StatefulIndicator` trait

**Important**: Each indicator implements EITHER vectorized OR stateful, not both. All indicators generate stateful MQL5 code via `generate_mql5()`.

### Type Safety

The type system enforces:
- Correct number of arguments (`arity()`)
- Correct input types (`input_types()`)
- Correct output type (`output_type()`)
- Semantic meaning (`scale_type()`)

This prevents nonsensical indicator compositions during genetic algorithm evolution.

## Next Steps

Proceed to **[03-primitives.md](./03-primitives.md)** to implement the 12 primitive building blocks.
