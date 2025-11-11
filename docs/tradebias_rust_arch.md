# TradeBias: Complete Rust Architecture

## 1. Module Structure & Organization

### Recommended Directory Layout

```
tradebias/
├── src/
│   ├── lib.rs                           # Library root, public API
│   ├── bin/
│   │   └── tradebias.rs                 # CLI entry point (optional)
│   ├── error.rs                         # Error types (thiserror)
│   ├── types.rs                         # Core type definitions
│   ├── config/
│   │   ├── mod.rs
│   │   ├── manager.rs                   # Configuration singleton
│   │   └── schema.rs                    # Config schema (serde)
│   ├── data/
│   │   ├── mod.rs
│   │   ├── types.rs                     # Trade, StrategyResult, etc.
│   │   ├── connectors/
│   │   │   ├── mod.rs
│   │   │   ├── csv.rs
│   │   │   ├── supabase.rs
│   │   │   └── memory.rs                # For testing
│   │   └── loader.rs                    # Data validation & normalization
│   ├── functions/
│   │   ├── mod.rs
│   │   ├── registry.rs                  # Function discovery
│   │   ├── indicators/
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs                # Indicator trait
│   │   │   ├── trend/                   # SMA, EMA, etc.
│   │   │   ├── momentum/                # RSI, MACD, etc.
│   │   │   ├── volatility/              # ATR, etc.
│   │   │   └── volume/                  # OBV, etc.
│   │   ├── primitives/
│   │   │   ├── mod.rs
│   │   │   ├── actions.rs               # OpenLong, ClosePosition
│   │   │   ├── comparisons.rs           # GreaterThan, etc.
│   │   │   ├── logical.rs               # And, Or, Not
│   │   │   └── math.rs                  # Add, Subtract, etc.
│   │   └── risk/
│   │       ├── mod.rs
│   │       ├── stop_loss.rs
│   │       └── take_profit.rs
│   ├── engines/
│   │   ├── mod.rs
│   │   ├── generation/
│   │   │   ├── mod.rs
│   │   │   ├── ast.rs                   # AST types & builders
│   │   │   ├── genome.rs                # Genome representation
│   │   │   ├── semantic_mapper.rs       # Type-driven AST generation
│   │   │   ├── evaluator.rs             # Fitness evaluation loop
│   │   │   └── operators.rs             # Mutation, crossover
│   │   ├── evaluation/
│   │   │   ├── mod.rs
│   │   │   ├── backtester.rs            # Orchestrator
│   │   │   ├── portfolio.rs             # Trade simulation
│   │   │   └── expression.rs            # AST → Polars expressions
│   │   ├── metrics/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs                # Calculation orchestration
│   │   │   ├── profitability.rs         # Profit-related metrics
│   │   │   ├── risk.rs                  # Drawdown, volatility
│   │   │   ├── returns.rs               # Sharpe, Sortino, etc.
│   │   │   └── trades.rs                # Win rate, etc.
│   │   └── validation/
│   │       ├── mod.rs
│   │       ├── robustness.rs            # Monte Carlo, friction
│   │       └── semantic.rs              # Type checking, diversity
│   ├── ml/
│   │   ├── mod.rs
│   │   ├── features/
│   │   │   ├── mod.rs
│   │   │   ├── engineer.rs              # Feature creation
│   │   │   └── types.rs
│   │   ├── labeling/
│   │   │   ├── mod.rs
│   │   │   └── triple_barrier.rs
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── base.rs                  # Trait definitions
│   │       ├── ensemble.rs              # Ensemble combining
│   │       └── trainer.rs               # Training orchestration
│   └── utils/
│       ├── mod.rs
│       ├── ast_converter.rs             # AST → human-readable
│       ├── code_gen.rs                  # AST → MQL5/Python code
│       └── metadata.rs                  # Indicator/scale metadata
├── tests/
│   ├── integration_tests.rs
│   ├── fixtures/                        # Test data
│   └── golden/                          # Golden file tests
├── Cargo.toml
├── Cargo.lock
└── README.md
```

### Dependency Graph (What Depends On What)

```
┌─────────────────────────────────────────────────────────┐
│                   egui UI (Frontend)                     │
│         (calls public API from engines/)                 │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│         Public Crate API (lib.rs)                        │
│  • run_evolution()                                       │
│  • run_backtest()                                        │
│  • train_ml_model()                                      │
│  • calculate_metrics()                                   │
└────────────────┬────────────────────────────────────────┘
                 │
    ┌────────────┼────────────────┐
    │            │                │
┌───▼──┐  ┌─────▼────┐  ┌────────▼────┐
│engines│  │functions │  │     ml/     │
│       │  │registry  │  │  features   │
└───┬──┘  └─────┬────┘  └────────┬────┘
    │           │                │
    │      ┌────▼───────────┐   │
    │      │ Trait Catalog  │◄──┘
    │      │ • Indicator    │
    │      │ • Metric       │
    │      │ • StratFunc    │
    │      └────┬───────────┘
    │           │
    │      ┌────▼──────────────┐
    │      │   functions/      │
    │      │   indicators/     │
    │      │   primitives/     │
    │      └───────┬───────────┘
    │             │
    │      ┌──────▼────────┐
    │      │  data/types   │
    │      │  config/      │
    │      │  error.rs     │
    │      └───────────────┘
    │
    ├─► data/ (connectors, loaders)
    │
    ├─► functions/ (indicators, primitives, risk)
    │
    ├─► config/ (manager, schema)
    │
    └─► error/ (error types)

NO CIRCULAR DEPENDENCIES - DAG structure enforced by module organization
```

## 2. Trait Hierarchy & Type System

### Core Trait Architecture

```rust
// src/types.rs - Foundational types

/// Value scale information (compile-time semantic info)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Price,                    // Follows price (SMA, Bollinger Bands)
    Oscillator0_100,          // Bounded 0-100 (RSI, Stochastic)
    OscillatorCentered,       // Zero-centered (MACD, Momentum)
    Volatility,               // Small decimals (ATR, StdDev)
    Volume,                   // Large integers (OBV)
    Ratio,                    // Ratios (Williams %R)
}

/// Semantic type for strategy expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    NumericSeries,            // Polars Series<f64>
    BoolSeries,               // Polars Series<bool>
    Integer,                  // Scalar i32
    Float,                    // Scalar f64
}

// src/functions/indicators/traits.rs

/// Base indicator trait - all indicators implement this
pub trait Indicator: Send + Sync {
    /// Unique identifier (e.g., "SMA", "RSI")
    fn alias(&self) -> &'static str;
    
    /// Display name for UI
    fn ui_name(&self) -> &'static str;
    
    /// Semantic scale type (for validation)
    fn scale_type(&self) -> ScaleType;
    
    /// Expected value range (e.g., 0-100 for RSI)
    fn value_range(&self) -> Option<(f64, f64)>;
    
    /// Number of required parameters
    fn arity(&self) -> usize;
    
    /// Input data type for each parameter
    fn input_types(&self) -> Vec<DataType>;
    
    /// Output type (always NumericSeries for indicators)
    fn output_type(&self) -> DataType {
        DataType::NumericSeries
    }
    
    /// Execute indicator (returns Polars expression)
    fn call(&self, args: &[IndicatorArg]) -> Result<Expr>;
    
    /// Generate MQL5 code
    fn generate_mql5(&self, args: &[String]) -> String;
}

/// Flexible argument type for indicator calls
#[derive(Debug, Clone)]
pub enum IndicatorArg {
    Series(Expr),             // For price series
    Scalar(f64),              // For periods, thresholds
}

// src/functions/primitives/mod.rs

/// Primitive function trait (operators, actions, etc.)
pub trait PrimitiveFunc: Send + Sync {
    fn alias(&self) -> &'static str;
    fn ui_name(&self) -> &'static str;
    fn arity(&self) -> usize;
    fn input_types(&self) -> Vec<DataType>;
    fn output_type(&self) -> DataType;
    fn call(&self, args: &[Expr]) -> Result<Expr>;
    fn generate_mql5(&self, args: &[String]) -> String;
}

// src/engines/metrics/traits.rs

/// Generic metric calculator
pub trait MetricCalculator: Send + Sync {
    /// Metric identifier (e.g., "sharpe_ratio")
    fn id(&self) -> &'static str;
    
    /// Human-readable name
    fn display_name(&self) -> &'static str;
    
    /// What this metric depends on (for topological sort)
    fn dependencies(&self) -> Vec<&'static str>;
    
    /// Calculate metric given dependencies
    fn calculate(
        &self,
        context: &MetricContext,
    ) -> Result<f64>;
}

/// Context passed to metric calculators
pub struct MetricContext {
    pub trades: Vec<Trade>,
    pub equity_curve: Series,
    pub risk_free_rate: f64,
    pub periods_per_year: f64,
    // Additional cached metrics for dependencies
    pub cache: HashMap<String, f64>,
}

// src/ml/models/base.rs

/// ML model trait for ensemble
pub trait MLModel: Send + Sync {
    /// Predict probability for binary classification
    fn predict_proba(&self, features: &DataFrame) -> Result<Vec<f64>>;
    
    /// Feature importance scores
    fn feature_importance(&self) -> Vec<f64>;
}

// src/engines/generation/ast.rs

/// Abstract Syntax Tree - represents a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    /// Constant value (integer, float, etc.)
    Const(Value),
    
    /// Function call with arguments
    Call {
        function: String,
        args: Vec<Box<AstNode>>,
    },
    
    /// Complete strategy rule
    Rule {
        condition: Box<AstNode>,
        action: Box<AstNode>,
    },
}

impl AstNode {
    /// Type-check this node given a function registry
    pub fn type_check(&self, registry: &FunctionRegistry) -> Result<DataType>;
    
    /// Check AST structural validity
    pub fn validate(&self) -> Result<()>;
    
    /// Generate canonical string for deduplication
    pub fn canonical_string(&self) -> String;
}
```

### Type-Safe Newtype Patterns

```rust
// src/data/types.rs - Use newtypes to encode semantics

/// A price value (e.g., from OHLC data)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Price(pub f64);

/// An oscillator value (0-100 like RSI)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Oscillator(pub f64);

/// Prevents "RSI > Close" at compile time:
impl PartialOrd<Oscillator> for Price {
    // This comparison doesn't exist! Type error caught at compile time
}

/// Position size as percentage of account
#[derive(Debug, Clone, Copy)]
pub struct PositionSize(pub f64); // 0.0-1.0

/// Maximum Drawdown percentage
#[derive(Debug, Clone, Copy)]
pub struct MaxDrawdown(pub f64);

// Usage:
let price_val = Price(100.5);
let osc_val = Oscillator(70.0);

// ✗ Compile error: PartialOrd not implemented between these types
// if price_val > osc_val { ... }

// ✓ This works:
if osc_val.0 > 70.0 { ... }
```

## 3. Concrete Implementation Examples

### Example 1: Indicator Implementation (SMA)

```rust
// src/functions/indicators/trend/sma.rs

use crate::functions::indicators::Indicator;
use crate::types::{DataType, ScaleType};
use polars::prelude::*;
use anyhow::Result;

/// Simple Moving Average
pub struct SMA {
    period: usize,
}

impl SMA {
    pub fn new() -> Self {
        Self { period: 14 }
    }
}

impl Indicator for SMA {
    fn alias(&self) -> &'static str {
        "SMA"
    }

    fn ui_name(&self) -> &'static str {
        "Simple Moving Average"
    }

    fn scale_type(&self) -> ScaleType {
        ScaleType::Price  // SMA follows price scale
    }

    fn value_range(&self) -> Option<(f64, f64)> {
        None  // SMA has no fixed range
    }

    fn arity(&self) -> usize {
        2  // Takes 2 arguments: series, period
    }

    fn input_types(&self) -> Vec<DataType> {
        vec![DataType::NumericSeries, DataType::Integer]
    }

    fn call(&self, args: &[IndicatorArg]) -> Result<Expr> {
        if args.len() != 2 {
            anyhow::bail!("SMA requires 2 arguments, got {}", args.len());
        }

        let series = match &args[0] {
            IndicatorArg::Series(expr) => expr.clone(),
            _ => anyhow::bail!("SMA: first argument must be a series"),
        };

        let period = match &args[1] {
            IndicatorArg::Scalar(val) => {
                if *val <= 0.0 || *val > 1000.0 {
                    anyhow::bail!("SMA period must be 1-1000, got {}", val);
                }
                *val as u32 as usize
            }
            _ => anyhow::bail!("SMA: second argument must be scalar"),
        };

        // Polars expression for rolling mean
        Ok(series.rolling_mean(RollingOptionsFixedWindow {
            window_size: period,
            ..Default::default()
        }))
    }

    fn generate_mql5(&self, args: &[String]) -> String {
        if args.len() < 2 {
            return "ERROR: SMA requires 2 args".to_string();
        }
        let period = &args[1];
        format!("iMA({}, {}, {}, MODE_SMA)", args[0], period, 0)
    }
}

// Register this indicator
impl IndicatorRegistry {
    pub fn register_sma(self) -> Self {
        self.register("SMA", Box::new(SMA::new()))
    }
}
```

### Example 2: Metric Implementation (Sharpe Ratio)

```rust
// src/engines/metrics/returns.rs

use crate::data::types::Trade;
use crate::engines::metrics::{MetricCalculator, MetricContext};
use anyhow::Result;

pub struct SharpeRatio;

impl MetricCalculator for SharpeRatio {
    fn id(&self) -> &'static str {
        "sharpe_ratio"
    }

    fn display_name(&self) -> &'static str {
        "Sharpe Ratio"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["returns", "volatility"]  // Depends on annual returns and vol
    }

    fn calculate(&self, context: &MetricContext) -> Result<f64> {
        // This assumes "returns" and "volatility" were already calculated
        let annual_returns = context
            .cache
            .get("returns")
            .ok_or_else(|| anyhow::anyhow!("Missing 'returns' dependency"))?;

        let volatility = context
            .cache
            .get("volatility")
            .ok_or_else(|| anyhow::anyhow!("Missing 'volatility' dependency"))?;

        if *volatility == 0.0 {
            return Err(anyhow::anyhow!("Volatility is zero, cannot calculate Sharpe"));
        }

        let risk_adjusted_return = (annual_returns - context.risk_free_rate) / volatility;
        Ok(risk_adjusted_return)
    }
}

// Example: Volatility metric (dependency for Sharpe)
pub struct AnnualVolatility;

impl MetricCalculator for AnnualVolatility {
    fn id(&self) -> &'static str {
        "volatility"
    }

    fn display_name(&self) -> &'static str {
        "Annual Volatility"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["equity_curve"]  // Only depends on raw equity curve
    }

    fn calculate(&self, context: &MetricContext) -> Result<f64> {
        let equity = context.equity_curve.f64()?;
        let values = equity.to_vec();
        
        if values.len() < 2 {
            return Err(anyhow::anyhow!("Insufficient data for volatility"));
        }

        // Calculate daily returns
        let mut returns = Vec::new();
        for i in 1..values.len() {
            if let (Some(prev), Some(curr)) = (values.get(i - 1), values.get(i)) {
                if prev > 0.0 {
                    returns.push((curr - prev) / prev);
                }
            }
        }

        if returns.is_empty() {
            return Err(anyhow::anyhow!("No valid returns calculated"));
        }

        // Calculate sample standard deviation
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / (returns.len() - 1) as f64;

        let daily_vol = variance.sqrt();
        let annual_vol = daily_vol * (context.periods_per_year).sqrt();

        Ok(annual_vol)
    }
}
```

### Example 3: Genetic Operator (Mutation)

```rust
// src/engines/generation/operators.rs

use crate::engines::generation::ast::AstNode;
use crate::functions::registry::FunctionRegistry;
use rand::Rng;
use anyhow::Result;

pub struct GeneticOperators {
    registry: FunctionRegistry,
}

impl GeneticOperators {
    pub fn new(registry: FunctionRegistry) -> Self {
        Self { registry }
    }

    /// Mutate an AST by replacing a random node
    pub fn mutate(
        &self,
        ast: &AstNode,
        mutation_rate: f64,
        rng: &mut impl Rng,
        max_depth: usize,
    ) -> Result<AstNode> {
        if rng.gen::<f64>() > mutation_rate {
            return Ok(ast.clone());
        }

        self.mutate_recursive(ast, rng, max_depth, 0)
    }

    fn mutate_recursive(
        &self,
        node: &AstNode,
        rng: &mut impl Rng,
        max_depth: usize,
        current_depth: usize,
    ) -> Result<AstNode> {
        if current_depth >= max_depth {
            return Ok(node.clone());
        }

        match node {
            AstNode::Const(val) => {
                // 50% chance to mutate constant values
                if rng.gen_bool(0.5) {
                    let new_val = match val {
                        Value::Integer(i) => {
                            let delta = rng.gen_range(-10..=10) as i64;
                            Value::Integer((*i + delta).max(0).min(1000))
                        }
                        Value::Float(f) => {
                            let delta = rng.gen_range(-1.0..=1.0);
                            Value::Float((f + delta).max(0.0).min(100.0))
                        }
                        _ => val.clone(),
                    };
                    Ok(AstNode::Const(new_val))
                } else {
                    Ok(node.clone())
                }
            }

            AstNode::Call { function, args } => {
                // 30% chance to replace this call node entirely
                if rng.gen_bool(0.3) && current_depth < max_depth - 1 {
                    // Generate new node of compatible type
                    let new_node = self.generate_compatible_node(
                        node.type_check(&self.registry)?,
                        rng,
                        max_depth,
                        current_depth + 1,
                    )?;
                    return Ok(new_node);
                }

                // Otherwise, mutate arguments
                let mutated_args = args
                    .iter()
                    .map(|arg| {
                        Box::new(self.mutate_recursive(
                            arg,
                            rng,
                            max_depth,
                            current_depth + 1,
                        )?)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(AstNode::Call {
                    function: function.clone(),
                    args: mutated_args,
                })
            }

            AstNode::Rule {
                condition,
                action,
            } => {
                // Mutate condition and action independently
                let new_condition =
                    Box::new(self.mutate_recursive(condition, rng, max_depth, current_depth + 1)?);
                let new_action =
                    Box::new(self.mutate_recursive(action, rng, max_depth, current_depth + 1)?);

                Ok(AstNode::Rule {
                    condition: new_condition,
                    action: new_action,
                })
            }
        }
    }

    /// Crossover: swap subtrees between two strategies
    pub fn crossover(
        &self,
        parent1: &AstNode,
        parent2: &AstNode,
        rng: &mut impl Rng,
    ) -> Result<(AstNode, AstNode)> {
        let point1 = self.random_crossover_point(parent1, rng);
        let point2 = self.random_crossover_point(parent2, rng);

        let child1 = self.swap_subtree(parent1, parent2, point1)?;
        let child2 = self.swap_subtree(parent2, parent1, point2)?;

        Ok((child1, child2))
    }

    fn random_crossover_point(
        &self,
        node: &AstNode,
        rng: &mut impl Rng,
    ) -> usize {
        // In real implementation, select random node index
        rng.gen_range(0..self.count_nodes(node))
    }

    fn count_nodes(&self, node: &AstNode) -> usize {
        match node {
            AstNode::Const(_) => 1,
            AstNode::Call { args, .. } => 1 + args.iter().map(|a| self.count_nodes(a)).sum::<usize>(),
            AstNode::Rule {
                condition,
                action,
            } => 1 + self.count_nodes(condition) + self.count_nodes(action),
        }
    }

    fn swap_subtree(
        &self,
        dest: &AstNode,
        source: &AstNode,
        point: usize,
    ) -> Result<AstNode> {
        // Implementation: replace node at 'point' in 'dest' with subtree from 'source'
        Ok(dest.clone()) // Simplified for brevity
    }

    fn generate_compatible_node(
        &self,
        expected_type: DataType,
        rng: &mut impl Rng,
        max_depth: usize,
        current_depth: usize,
    ) -> Result<AstNode> {
        // This would use SemanticMapper to generate type-valid nodes
        todo!()
    }
}
```

### Example 4: Pipeline Stage (Feature Engineer)

```rust
// src/ml/features/engineer.rs

use polars::prelude::*;
use anyhow::Result;

/// Feature engineer for ML signal filtering
pub struct FeatureEngineer {
    lookback: usize,
}

impl FeatureEngineer {
    pub fn new(lookback: usize) -> Self {
        Self { lookback }
    }

    /// Engineer features from signals and market data
    pub fn engineer(
        &self,
        signals: &DataFrame,
        market_data: &DataFrame,
    ) -> Result<DataFrame> {
        let mut features = signals.clone();

        // Price-based features
        features = self.add_price_features(&features, market_data)?;

        // Momentum features
        features = self.add_momentum_features(&features)?;

        // Volatility features
        features = self.add_volatility_features(&features)?;

        Ok(features)
    }

    fn add_price_features(
        &self,
        df: &DataFrame,
        market_data: &DataFrame,
    ) -> Result<DataFrame> {
        let mut features = df.clone();

        // Returns over different windows
        let close = market_data.column("close")?;
        for window in &[1, 5, 20] {
            let returns = close
                .f64()?
                .into_iter()
                .collect::<Vec<_>>();

            let mut return_values = Vec::new();
            for i in *window..returns.len() {
                if let (Some(prev), Some(curr)) = (returns.get(i - window), returns.get(i)) {
                    if let (Some(p), Some(c)) = (prev, curr) {
                        return_values.push(Some((c - p) / p));
                    }
                }
            }

            features = features.with_column(
                Series::new(
                    &format!("return_{}", window),
                    return_values,
                )
            )?;
        }

        Ok(features)
    }

    fn add_momentum_features(&self, df: &DataFrame) -> Result<DataFrame> {
        // RSI, MACD, etc. as features
        let mut features = df.clone();

        // Simple momentum (12-period / 26-period)
        if let Ok(col) = df.column("close") {
            // Feature engineering logic here
        }

        Ok(features)
    }

    fn add_volatility_features(&self, df: &DataFrame) -> Result<DataFrame> {
        // Standard deviation, ATR, etc.
        Ok(df.clone())
    }
}

/// Trait for pipeline stages (extensible)
pub trait PipelineStage: Send + Sync {
    fn execute(&self, input: &DataFrame) -> Result<DataFrame>;
    fn stage_name(&self) -> &'static str;
}

impl PipelineStage for FeatureEngineer {
    fn execute(&self, input: &DataFrame) -> Result<DataFrame> {
        // For standalone use
        self.engineer(input, &self.get_market_data()?)
    }

    fn stage_name(&self) -> &'static str {
        "feature_engineer"
    }
}
```

### Example 5: Error Handling

```rust
// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradebiasError {
    // Semantic/validation errors (domain-specific)
    #[error("Invalid strategy AST: {0}")]
    InvalidAst(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid indicator parameter: {0}")]
    InvalidIndicatorParam(String),

    #[error("Backtest failed: {0}")]
    BacktestError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Metric dependency cycle detected")]
    DependencyCycle,

    // I/O errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    CsvError(String),

    #[error("Data connector error: {0}")]
    ConnectorError(String),

    // External service errors
    #[error("Supabase error: {0}")]
    SupabaseError(String),

    #[error("Model training failed: {0}")]
    ModelTrainingError(String),

    // Generic errors (internal)
    #[error("Internal error: {0}")]
    Internal(String),

    // Polars errors
    #[error("Polars operation failed: {0}")]
    Polars(#[from] polars::error::PolarsError),
}

pub type Result<T> = std::result::Result<T, TradebiasError>;

// Usage in code:
pub fn calculate_metric(data: &DataFrame) -> Result<f64> {
    if data.height() == 0 {
        return Err(TradebiasError::InsufficientData(
            "Empty DataFrame".to_string(),
        ));
    }

    // ... calculation ...
    Ok(result)
}

// Error propagation with context:
pub fn run_backtest(ast: &AstNode, data: &DataFrame) -> Result<StrategyResult> {
    let expr = build_expression(ast)
        .map_err(|e| TradebiasError::BacktestError(format!("Expression build failed: {}", e)))?;

    // ... rest of backtest ...
    Ok(result)
}
```

## 4. Testing Patterns

### Unit Test Example

```rust
// src/functions/indicators/trend/sma.rs - tests module

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_sma_basic() {
        let sma = SMA::new();

        // Create test data
        let data = Series::new("price", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let expr = Expr::Literal(LiteralValue::Series(data));

        let period = IndicatorArg::Scalar(3.0);
        let args = vec![IndicatorArg::Series(expr), period];

        let result = sma.call(&args).unwrap();
        // Assert result...
    }

    #[test]
    fn test_sma_invalid_period() {
        let sma = SMA::new();
        let bad_period = IndicatorArg::Scalar(-1.0);

        let result = sma.call(&[
            IndicatorArg::Series(col("close")),
            bad_period,
        ]);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("period"));
    }

    #[test]
    fn test_sma_metadata() {
        let sma = SMA::new();
        assert_eq!(sma.alias(), "SMA");
        assert_eq!(sma.scale_type(), ScaleType::Price);
        assert_eq!(sma.arity(), 2);
    }
}
```

### Integration Test Example

```rust
// tests/integration_tests.rs

use tradebias::engines::evaluation::Backtester;
use tradebias::engines::generation::SemanticMapper;
use tradebias::data::types::StrategyResult;
use polars::prelude::*;

#[test]
fn test_full_evolution_cycle() {
    // Setup
    let data = load_test_data().unwrap();
    let registry = setup_function_registry();
    let backtester = Backtester::new(&registry);
    let mapper = SemanticMapper::new(&registry);

    // Generate strategy from genome
    let genome = vec![0, 1, 14, 2, 70, 3];  // Encoded strategy
    let ast = mapper.create_strategy_ast(&genome).unwrap();

    // Validate AST type-checking
    assert!(ast.type_check(&registry).is_ok());

    // Run backtest
    let result = backtester.run(&ast, &data).unwrap();

    // Verify results structure
    assert!(result.metrics.contains_key("sharpe_ratio"));
    assert!(!result.trades.is_empty() || result.metrics.get("win_rate").unwrap() > &0.0);
    assert!(!result.equity_curve.is_empty());
}

#[test]
fn test_mutation_preserves_validity() {
    let registry = setup_function_registry();
    let mut rng = rand::thread_rng();
    let operators = GeneticOperators::new(registry.clone());

    for _ in 0..100 {
        let genome = generate_random_genome(&mut rng);
        let ast = SemanticMapper::new(&registry)
            .create_strategy_ast(&genome)
            .unwrap();

        let mutated = operators.mutate(&ast, 0.5, &mut rng, 10).unwrap();

        // Mutated AST must still be valid
        assert!(mutated.type_check(&registry).is_ok(), "Mutation produced invalid AST");
    }
}

fn load_test_data() -> Result<DataFrame> {
    let data = vec![
        (1000.0, 1010.0, 995.0, 1005.0, 1_000_000.0),
        (1005.0, 1015.0, 1000.0, 1012.0, 950_000.0),
        // ... more OHLCV data
    ];

    let mut df = DataFrame::new(vec![
        Series::new("open", data.iter().map(|(o, _, _, _, _)| o).copied().collect::<Vec<_>>()),
        Series::new("high", data.iter().map(|(_, h, _, _, _)| h).copied().collect::<Vec<_>>()),
        Series::new("low", data.iter().map(|(_, _, l, _, _)| l).copied().collect::<Vec<_>>()),
        Series::new("close", data.iter().map(|(_, _, _, c, _)| c).copied().collect::<Vec<_>>()),
        Series::new("volume", data.iter().map(|(_, _, _, _, v)| v).copied().collect::<Vec<_>>()),
    ])?;

    Ok(df)
}
```

## 5. Error Handling & Recovery Strategy

### Graceful Degradation Pattern

```rust
// src/engines/metrics/engine.rs

pub struct MetricsEngine {
    calculators: HashMap<String, Arc<dyn MetricCalculator>>,
}

impl MetricsEngine {
    /// Calculate metrics, skipping those with errors (graceful degradation)
    pub fn calculate_all(&self, context: &MetricContext) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let mut cache = context.cache.clone();

        // Topologically sort metrics by dependencies
        let sorted = self.topological_sort().unwrap_or_default();

        for metric_id in sorted {
            match self.calculators.get(&metric_id) {
                Some(calc) => {
                    let mut ctx = context.clone();
                    ctx.cache = cache.clone();

                    match calc.calculate(&ctx) {
                        Ok(value) => {
                            results.insert(metric_id.clone(), value);
                            cache.insert(metric_id, value);
                            tracing::debug!("Calculated metric: {} = {}", metric_id, value);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to calculate metric {}: {}",
                                metric_id,
                                e
                            );
                            // Continue with other metrics instead of failing completely
                        }
                    }
                }
                None => {
                    tracing::warn!("Metric calculator not found: {}", metric_id);
                }
            }
        }

        results
    }

    fn topological_sort(&self) -> Result<Vec<String>> {
        // Build dependency graph and sort using Kahn's algorithm
        // Returns metrics in order of dependencies
        todo!()
    }
}
```

## 6. Performance Optimization Guidelines

### Parallelization with Rayon

```rust
// src/engines/generation/evaluator.rs

use rayon::prelude::*;

pub struct EvolutionEvaluator {
    backtester: Arc<Backtester>,
    metrics_engine: Arc<MetricsEngine>,
}

impl EvolutionEvaluator {
    /// Evaluate population in parallel
    pub fn evaluate_population(
        &self,
        population: &[AstNode],
        data: &DataFrame,
    ) -> Result<Vec<FitnessScore>> {
        population
            .par_iter()  // Parallel iterator
            .map(|strategy| {
                let result = self.backtester.run(strategy, data)?;
                let fitness = self.calculate_fitness(&result)?;
                Ok(fitness)
            })
            .collect()
    }

    fn calculate_fitness(&self, result: &StrategyResult) -> Result<FitnessScore> {
        // Multi-objective fitness
        let sharpe = result.metrics.get("sharpe_ratio").copied().unwrap_or(0.0);
        let win_rate = result.metrics.get("win_rate").copied().unwrap_or(0.0);
        let profit = result.metrics.get("net_profit").copied().unwrap_or(0.0);

        Ok(FitnessScore {
            sharpe,
            win_rate,
            profit,
            combined: (sharpe * 0.4 + win_rate * 0.3 + (profit / 10000.0).min(1.0) * 0.3),
        })
    }
}
```

### Memory-Efficient AST Operations

```rust
// Avoid unnecessary clones with Cow (Copy-on-Write)

use std::borrow::Cow;

pub fn mutate_node(node: &AstNode, mutation_rate: f64) -> Cow<AstNode> {
    if should_mutate(mutation_rate) {
        Cow::Owned(perform_mutation(node))
    } else {
        Cow::Borrowed(node)  // Zero-copy when no mutation needed
    }
}
```

## 7. Frontend Integration (egui)

### Direct Function Calls (No JSON)

```rust
// ui/src/app/mod.rs

use tradebias::engines::generation::EvolutionConfig;
use tradebias::engines::evaluation::Backtester;

pub struct MyApp {
    // Direct access to Rust backend - no IPC needed!
    backtester: Arc<Backtester>,
    evolution_config: EvolutionConfig,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Run Evolution").clicked() {
                // Direct function call!
                match self.run_evolution() {
                    Ok(strategies) => {
                        self.hall_of_fame = strategies;
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                    }
                }
            }
        });
    }
}

impl MyApp {
    fn run_evolution(&mut self) -> Result<Vec<StrategyResult>> {
        // No JSON serialization!
        // Direct Rust function calls with type safety
        let genome = generate_random_genome();
        let result = self.backtester.run(&genome, &self.data)?;
        Ok(vec![result])
    }
}
```

## 8. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Error types and result handling (`error.rs`)
- [ ] Core type definitions (`types.rs`)
- [ ] Trait definitions (indicators, metrics, primitives)
- [ ] AST representation and validation

### Phase 2: Core Engines (Week 2)
- [ ] Indicator library (SMA, RSI, basic set)
- [ ] Expression builder (AST → Polars)
- [ ] Portfolio simulator (trade execution)
- [ ] Metrics engine (5-10 core metrics)

### Phase 3: Evolution (Week 3)
- [ ] Semantic mapper (genome → AST)
- [ ] Genetic operators (mutation, crossover)
- [ ] Evolution loop (full generation cycle)
- [ ] Hall of Fame management

### Phase 4: Integration (Week 4)
- [ ] Remove Python backend
- [ ] Remove IPC layer
- [ ] Direct egui → Rust engine calls
- [ ] Single-binary compilation

### Phase 5: ML Pipeline (Weeks 5-6)
- [ ] Feature engineering
- [ ] Triple-barrier labeling
- [ ] Model training (smartcore)
- [ ] Signal filtering

### Phase 6: Polish (Week 7)
- [ ] Comprehensive testing
- [ ] Performance profiling & optimization
- [ ] Documentation
- [ ] Release build & optimization

## 9. Key Design Principles

### Principle 1: Semantic Validity at Compile Time
Use Rust's type system to prevent invalid strategies. Impossible states should be unrepresentable in the type system.

### Principle 2: Zero-Copy Wherever Possible
Use references, Cow, and borrowing to avoid unnecessary allocations during the hottest loops (backtesting, evolution).

### Principle 3: Trait-Based Extensibility
New indicators, metrics, models should be added without modifying existing code. Use trait impls and registry pattern.

### Principle 4: Explicit Error Handling
Never silently fail. Use `Result` types throughout. Log warnings for recoverable errors (e.g., metric calculation failures).

### Principle 5: Separation of Concerns
- `data/`: Loading, validation, types
- `functions/`: Indicators, operators, risk functions
- `engines/`: Evolution, backtesting, metrics
- `ml/`: Feature engineering, labeling, models
- `config/`: Settings management
- `utils/`: Helper functions, code generation

## 10. Recommended Cargo.toml

```toml
[package]
name = "tradebias"
version = "1.0.0"
edition = "2021"

[dependencies]
# Data processing
polars = { version = "0.19", features = ["lazy", "csv", "json", "lazy_compilation"] }
arrow = "51"

# Numerical computing
ndarray = "0.15"
nalgebra = "0.33"
statrs = "0.16"

# Genetic programming & randomness
rand = "0.8"
rand_distr = "0.4"

# Machine learning
smartcore = { version = "0.3", features = ["serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Configuration
config = "0.13"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time handling
chrono = "0.4"

# Frontend
egui = "0.24"
eframe = "0.24"

# Parallelization
rayon = "1.7"

# HTTP (optional, for Supabase/API)
reqwest = { version = "0.11", features = ["json"], optional = true }
tokio = { version = "1", features = ["full"], optional = true }

[features]
default = ["api"]
api = ["reqwest", "tokio"]

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"
tempfile = "3"

[[bench]]
name = "evolution"
harness = false
```

---

## Summary: Why This Architecture Works

1. **Type Safety**: Newtypes, trait bounds, and enum ASTs prevent invalid strategies at compile time
2. **Performance**: Rayon parallelization, zero-copy borrowing, vectorized Polars operations
3. **Maintainability**: Trait-based design makes adding new indicators/metrics trivial
4. **Testability**: Pure functions with Result types, no IPC to mock
5. **Extensibility**: Open/Closed Principle: new features via trait impls, not code modifications
6. **Single Binary**: No Python dependency, no IPC overhead, simple deployment
7. **Developer Experience**: Rust compiler catches errors early; type hints guide API usage