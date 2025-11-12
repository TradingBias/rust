# 12 - Semantic Generation & Type-Driven AST Creation

## Goal
Implement the semantic mapper that generates syntactically and semantically valid strategy ASTs from genomes using type-driven recursive generation. This ensures all generated strategies are executable and meaningful.

## Prerequisites
- **02-type-system.md** - Core types
- **03-primitives.md** - Building blocks
- **04-06** - Indicators and registry
- **11-evolution-engine.md** - Genome representation

## What You'll Create
1. `SemanticMapper` - Type-aware AST generator
2. `GeneConsumer` - Deterministic gene consumption from genome
3. Type compatibility checking
4. Smart parameter generation with indicator metadata
5. Argument diversity validation
6. Lightweight AST validation

## Core Problem

**Naive Approach (BAD)**:
```rust
// Random generation - produces invalid strategies
let ast = random_tree(); // Might generate: "RSI > Close" (comparing oscillator to price)
```

**Semantic Approach (GOOD)**:
```rust
// Type-driven generation - ensures validity
let ast = semantic_mapper.create_strategy_ast(genome);
// Always generates valid: "RSI(Close, 14) > 70" (comparing oscillator to threshold)
```

## Type System Review

From the project architecture, we have these key types:

```rust
// From 02-type-system.md
pub enum DataType {
    NumericSeries,  // Polars Series (prices, indicators)
    BoolSeries,     // Polars boolean Series (conditions)
    Integer,        // Scalar integer (periods, thresholds)
    Float,          // Scalar float (multipliers, percentages)
    Generic,        // Flexible type
}

pub trait StrategyFunction {
    fn alias(&self) -> &str;
    fn arity(&self) -> usize;
    fn input_types(&self) -> &[DataType];
    fn output_type(&self) -> DataType;
}
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│             Genome (List of Integers)                   │
│        [42, 17, 88, 3, 45, 12, 99, ...]                 │
└────────────────────┬────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────┐
│               GeneConsumer                               │
│  • Deterministically consumes genes                     │
│  • Maps gene values to choices                          │
│  • Tracks position in genome                            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────┐
│             SemanticMapper                               │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │  1. Start with desired output type             │    │
│  │     (e.g., BoolSeries for condition)           │    │
│  └────────────────────────────────────────────────┘    │
│                      ↓                                   │
│  ┌────────────────────────────────────────────────┐    │
│  │  2. Get compatible functions                   │    │
│  │     registry.get_by_output_type(BoolSeries)    │    │
│  │     → [GreaterThan, LessThan, And, Or, ...]    │    │
│  └────────────────────────────────────────────────┘    │
│                      ↓                                   │
│  ┌────────────────────────────────────────────────┐    │
│  │  3. Consume gene to select function            │    │
│  │     gene % num_functions → pick GreaterThan    │    │
│  └────────────────────────────────────────────────┘    │
│                      ↓                                   │
│  ┌────────────────────────────────────────────────┐    │
│  │  4. Recursively build arguments                │    │
│  │     GreaterThan needs: [NumericSeries, Number] │    │
│  │     ├─> Arg 0: build_expr(NumericSeries)       │    │
│  │     │   → RSI(Close, 14)                        │    │
│  │     └─> Arg 1: build_expr(Number)              │    │
│  │         → 70                                    │    │
│  └────────────────────────────────────────────────┘    │
│                      ↓                                   │
│  ┌────────────────────────────────────────────────┐    │
│  │  5. Return complete AST                         │    │
│  │     GreaterThan(RSI(Close, 14), 70)            │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## Implementation

### Step 1: Gene Consumer

Create `src/engines/generation/gene_consumer.rs`:

```rust
/// Deterministically consumes genes from a genome
pub struct GeneConsumer<'a> {
    genome: &'a [u32],
    position: usize,
}

impl<'a> GeneConsumer<'a> {
    pub fn new(genome: &'a [u32]) -> Self {
        Self { genome, position: 0 }
    }

    /// Consume next gene and return value
    pub fn consume(&mut self) -> u32 {
        if self.position >= self.genome.len() {
            // Wrap around if genome exhausted
            self.position = 0;
        }

        let gene = self.genome[self.position];
        self.position += 1;
        gene
    }

    /// Consume gene and map to choice index
    pub fn choose(&mut self, num_choices: usize) -> usize {
        if num_choices == 0 {
            return 0;
        }
        (self.consume() as usize) % num_choices
    }

    /// Consume gene and map to integer range
    pub fn int_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u32;
        min + (self.consume() % range) as i32
    }

    /// Consume gene and map to float range
    pub fn float_range(&mut self, min: f64, max: f64) -> f64 {
        if min >= max {
            return min;
        }
        let gene = self.consume();
        let normalized = (gene as f64) / (u32::MAX as f64); // 0.0 to 1.0
        min + normalized * (max - min)
    }

    /// Check if genes remaining
    pub fn has_genes(&self) -> bool {
        self.position < self.genome.len()
    }

    pub fn position(&self) -> usize {
        self.position
    }
}
```

### Step 2: Indicator Metadata for Smart Parameters

Create `src/utils/indicator_metadata.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetadata {
    pub full_name: String,
    pub scale: ScaleType,
    pub value_range: Option<(f64, f64)>,
    pub category: String,
    pub typical_periods: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScaleType {
    Price,              // Follows price (SMA, EMA, Bollinger)
    Oscillator0_100,    // 0-100 range (RSI, Stochastic)
    OscillatorCentered, // Zero-centered (MACD, Momentum)
    VolatilityDecimal,  // Small decimals (ATR, StdDev)
    Volume,             // Large integers (OBV, Volume)
    Ratio,              // Ratio-based (Williams %R)
    Index,              // Index-based (ADX, CCI)
}

pub struct MetadataRegistry {
    metadata: HashMap<String, IndicatorMetadata>,
}

impl MetadataRegistry {
    pub fn new() -> Self {
        let mut metadata = HashMap::new();

        // Trend indicators
        metadata.insert(
            "SMA".to_string(),
            IndicatorMetadata {
                full_name: "Simple Moving Average".to_string(),
                scale: ScaleType::Price,
                value_range: None, // Follows price
                category: "trend".to_string(),
                typical_periods: Some(vec![5, 10, 14, 20, 50, 100, 200]),
            },
        );

        metadata.insert(
            "RSI".to_string(),
            IndicatorMetadata {
                full_name: "Relative Strength Index".to_string(),
                scale: ScaleType::Oscillator0_100,
                value_range: Some((0.0, 100.0)),
                category: "momentum".to_string(),
                typical_periods: Some(vec![9, 14, 21, 25]),
            },
        );

        metadata.insert(
            "ATR".to_string(),
            IndicatorMetadata {
                full_name: "Average True Range".to_string(),
                scale: ScaleType::VolatilityDecimal,
                value_range: Some((0.0, f64::MAX)),
                category: "volatility".to_string(),
                typical_periods: Some(vec![7, 14, 21]),
            },
        );

        // Add more indicators...

        Self { metadata }
    }

    pub fn get(&self, indicator: &str) -> Option<&IndicatorMetadata> {
        self.metadata.get(indicator)
    }

    /// Check if two indicators can be meaningfully compared
    pub fn are_compatible(&self, ind1: &str, ind2: &str) -> bool {
        match (self.get(ind1), self.get(ind2)) {
            (Some(meta1), Some(meta2)) => meta1.scale == meta2.scale,
            _ => false,
        }
    }

    /// Generate appropriate threshold for indicator
    pub fn generate_threshold(&self, indicator: &str, gene: u32) -> f64 {
        if let Some(meta) = self.get(indicator) {
            match meta.scale {
                ScaleType::Oscillator0_100 => {
                    // Common thresholds: 30, 70 (oversold/overbought)
                    let thresholds = [20.0, 30.0, 40.0, 60.0, 70.0, 80.0];
                    thresholds[(gene as usize) % thresholds.len()]
                }
                ScaleType::OscillatorCentered => {
                    // Zero-crossing or small thresholds
                    let thresholds = [-10.0, -5.0, 0.0, 5.0, 10.0];
                    thresholds[(gene as usize) % thresholds.len()]
                }
                ScaleType::VolatilityDecimal => {
                    // Small positive values
                    0.0001 + (gene as f64 / u32::MAX as f64) * 0.01
                }
                _ => (gene as f64 / u32::MAX as f64) * 100.0,
            }
        } else {
            (gene as f64 / u32::MAX as f64) * 100.0
        }
    }
}
```

### Step 3: Semantic Mapper

Create `src/engines/generation/semantic_mapper.rs`:

```rust
use crate::engines::generation::{ast::*, gene_consumer::GeneConsumer};
use crate::functions::{FunctionRegistry, DataType};
use crate::utils::indicator_metadata::MetadataRegistry;
use crate::error::TradeBiasError;

pub struct SemanticMapper {
    registry: FunctionRegistry,
    metadata: MetadataRegistry,
    max_depth: usize,
}

impl SemanticMapper {
    pub fn new(registry: FunctionRegistry, max_depth: usize) -> Self {
        Self {
            registry,
            metadata: MetadataRegistry::new(),
            max_depth,
        }
    }

    /// Main entry point: Create complete strategy AST from genome
    pub fn create_strategy_ast(&self, genome: &[u32]) -> Result<StrategyAST, TradeBiasError> {
        let mut consumer = GeneConsumer::new(genome);

        // Build condition (must return BoolSeries)
        let condition = self.build_expression(DataType::BoolSeries, &mut consumer, 0)?;

        // Build action (simple for now, can be extended)
        let action_choice = consumer.choose(2); // 0 = OpenLong, 1 = OpenShort
        let action = match action_choice {
            0 => ASTNode::Call {
                function: "OpenLong".to_string(),
                args: vec![],
            },
            _ => ASTNode::Call {
                function: "OpenShort".to_string(),
                args: vec![],
            },
        };

        Ok(StrategyAST::Rule {
            condition: Box::new(condition),
            action: Box::new(action),
        })
    }

    /// Recursively build expression of desired type
    fn build_expression(
        &self,
        desired_type: DataType,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<ASTNode, TradeBiasError> {
        // Depth limit to prevent infinite recursion
        if depth >= self.max_depth {
            return self.build_terminal(desired_type, consumer);
        }

        match desired_type {
            DataType::BoolSeries => self.build_bool_series(consumer, depth),
            DataType::NumericSeries => self.build_numeric_series(consumer, depth),
            DataType::Integer => self.build_integer(consumer),
            DataType::Float => self.build_float(consumer),
            DataType::Generic => self.build_numeric_series(consumer, depth),
        }
    }

    fn build_bool_series(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<ASTNode, TradeBiasError> {
        // Get functions that return BoolSeries
        let functions = self.registry.get_by_output_type(DataType::BoolSeries);

        if functions.is_empty() {
            return Err(TradeBiasError::Generation(
                "No functions return BoolSeries".to_string(),
            ));
        }

        // Choose function
        let func_idx = consumer.choose(functions.len());
        let func = &functions[func_idx];

        // Build arguments
        let args = self.build_arguments(func.input_types(), consumer, depth + 1)?;

        Ok(ASTNode::Call {
            function: func.alias().to_string(),
            args,
        })
    }

    fn build_numeric_series(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<ASTNode, TradeBiasError> {
        // Choice: indicator, primitive data accessor, or math operation
        let choice = consumer.choose(3);

        match choice {
            0 => self.build_indicator(consumer, depth),
            1 => self.build_data_accessor(consumer),
            _ => self.build_math_operation(consumer, depth),
        }
    }

    fn build_indicator(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<ASTNode, TradeBiasError> {
        let indicators = self.registry.get_indicators();

        if indicators.is_empty() {
            return self.build_data_accessor(consumer);
        }

        let func_idx = consumer.choose(indicators.len());
        let func = &indicators[func_idx];

        // Build arguments with smart parameter generation
        let args = self.build_indicator_arguments(func, consumer, depth + 1)?;

        Ok(ASTNode::Call {
            function: func.alias().to_string(),
            args,
        })
    }

    fn build_indicator_arguments(
        &self,
        func: &dyn StrategyFunction,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<Vec<ASTNode>, TradeBiasError> {
        let input_types = func.input_types();
        let mut args = Vec::new();

        for (i, &arg_type) in input_types.iter().enumerate() {
            match arg_type {
                DataType::Integer => {
                    // Smart period generation using metadata
                    if let Some(meta) = self.metadata.get(func.alias()) {
                        if let Some(periods) = &meta.typical_periods {
                            let period = periods[consumer.choose(periods.len())];
                            args.push(ASTNode::Const(ConstValue::Integer(period as i32)));
                        } else {
                            args.push(self.build_integer(consumer)?);
                        }
                    } else {
                        args.push(self.build_integer(consumer)?);
                    }
                }
                _ => {
                    args.push(self.build_expression(arg_type, consumer, depth)?);
                }
            }
        }

        Ok(args)
    }

    fn build_arguments(
        &self,
        input_types: &[DataType],
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<Vec<ASTNode>, TradeBiasError> {
        input_types
            .iter()
            .map(|&arg_type| self.build_expression(arg_type, consumer, depth))
            .collect()
    }

    fn build_data_accessor(&self, consumer: &mut GeneConsumer) -> Result<ASTNode, TradeBiasError> {
        let accessors = ["Open", "High", "Low", "Close", "Volume"];
        let choice = consumer.choose(accessors.len());

        Ok(ASTNode::Call {
            function: accessors[choice].to_string(),
            args: vec![],
        })
    }

    fn build_math_operation(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<ASTNode, TradeBiasError> {
        let operations = ["Add", "Subtract", "Multiply", "Divide"];
        let choice = consumer.choose(operations.len());

        let arg1 = self.build_numeric_series(consumer, depth + 1)?;
        let arg2 = self.build_numeric_series(consumer, depth + 1)?;

        Ok(ASTNode::Call {
            function: operations[choice].to_string(),
            args: vec![arg1, arg2],
        })
    }

    fn build_integer(&self, consumer: &mut GeneConsumer) -> Result<ASTNode, TradeBiasError> {
        // Common indicator periods
        let periods = [5, 7, 9, 10, 12, 14, 20, 21, 25, 30, 50, 100, 200];
        let value = periods[consumer.choose(periods.len())];

        Ok(ASTNode::Const(ConstValue::Integer(value)))
    }

    fn build_float(&self, consumer: &mut GeneConsumer) -> Result<ASTNode, TradeBiasError> {
        let value = consumer.float_range(0.0, 100.0);
        Ok(ASTNode::Const(ConstValue::Float(value)))
    }

    fn build_terminal(
        &self,
        desired_type: DataType,
        consumer: &mut GeneConsumer,
    ) -> Result<ASTNode, TradeBiasError> {
        match desired_type {
            DataType::NumericSeries => self.build_data_accessor(consumer),
            DataType::Integer => self.build_integer(consumer),
            DataType::Float => self.build_float(consumer),
            _ => Err(TradeBiasError::Generation(format!(
                "Cannot build terminal for type {:?}",
                desired_type
            ))),
        }
    }
}
```

### Step 4: Argument Diversity Validator

Create `src/engines/generation/diversity_validator.rs`:

```rust
use crate::engines::generation::ast::*;
use std::collections::{HashMap, HashSet};

/// Validates that indicator parameters are diverse
pub struct DiversityValidator {
    min_param_difference: i32,
}

impl DiversityValidator {
    pub fn new(min_param_difference: i32) -> Self {
        Self { min_param_difference }
    }

    /// Check if AST has diverse indicator parameters
    pub fn validate(&self, ast: &StrategyAST) -> bool {
        let mut indicator_params: HashMap<String, Vec<i32>> = HashMap::new();

        self.collect_indicator_params(ast, &mut indicator_params);

        // Check each indicator type
        for (_indicator, params) in indicator_params.iter() {
            if !self.are_params_diverse(params) {
                return false;
            }
        }

        true
    }

    fn collect_indicator_params(
        &self,
        node: &ASTNode,
        collector: &mut HashMap<String, Vec<i32>>,
    ) {
        match node {
            ASTNode::Call { function, args } => {
                // If this is an indicator call with integer params
                if args.len() > 0 {
                    if let Some(ASTNode::Const(ConstValue::Integer(period))) = args.get(1) {
                        collector
                            .entry(function.clone())
                            .or_insert_with(Vec::new)
                            .push(*period);
                    }
                }

                // Recurse into arguments
                for arg in args {
                    self.collect_indicator_params(arg, collector);
                }
            }
            ASTNode::Const(_) => {}
        }
    }

    fn are_params_diverse(&self, params: &[i32]) -> bool {
        if params.len() <= 1 {
            return true; // Single param is always diverse
        }

        // Check all pairs
        for i in 0..params.len() {
            for j in (i + 1)..params.len() {
                let diff = (params[i] - params[j]).abs();
                if diff < self.min_param_difference {
                    return false; // Too similar
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diversity_validator() {
        let validator = DiversityValidator::new(5);

        // Valid: SMA(14) and SMA(20) differ by 6
        let ast1 = create_test_ast_with_params("SMA", vec![14, 20]);
        assert!(validator.validate(&ast1));

        // Invalid: SMA(14) and SMA(15) differ by only 1
        let ast2 = create_test_ast_with_params("SMA", vec![14, 15]);
        assert!(!validator.validate(&ast2));
    }
}
```

### Step 5: Lightweight AST Validator

Create `src/engines/generation/lightweight_validator.rs`:

```rust
use crate::engines::generation::ast::*;
use crate::functions::FunctionRegistry;
use crate::error::TradeBiasError;

pub struct LightweightValidator {
    registry: FunctionRegistry,
    max_depth: usize,
}

impl LightweightValidator {
    pub fn new(registry: FunctionRegistry, max_depth: usize) -> Self {
        Self { registry, max_depth }
    }

    /// Validate AST structure, types, and arity
    pub fn validate(&self, ast: &StrategyAST) -> Result<(), TradeBiasError> {
        match ast {
            StrategyAST::Rule { condition, action } => {
                self.validate_node(condition, 0)?;
                self.validate_node(action, 0)?;
                Ok(())
            }
        }
    }

    fn validate_node(&self, node: &ASTNode, depth: usize) -> Result<(), TradeBiasError> {
        // Depth check
        if depth > self.max_depth {
            return Err(TradeBiasError::Validation(format!(
                "AST depth {} exceeds maximum {}",
                depth, self.max_depth
            )));
        }

        match node {
            ASTNode::Call { function, args } => {
                // Function exists?
                let func = self.registry.get_function(function).ok_or_else(|| {
                    TradeBiasError::Validation(format!("Unknown function: {}", function))
                })?;

                // Arity matches?
                if args.len() != func.arity() {
                    return Err(TradeBiasError::Validation(format!(
                        "Function {} expects {} args, got {}",
                        function,
                        func.arity(),
                        args.len()
                    )));
                }

                // Type compatibility (simplified check)
                let input_types = func.input_types();
                for (i, arg) in args.iter().enumerate() {
                    if i < input_types.len() {
                        // TODO: Deep type checking
                        self.validate_node(arg, depth + 1)?;
                    }
                }

                Ok(())
            }
            ASTNode::Const(_) => Ok(()), // Constants are always valid
        }
    }
}
```

## Usage Example

```rust
use tradebias::engines::generation::semantic_mapper::SemanticMapper;
use tradebias::functions::FunctionRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load function registry
    let registry = FunctionRegistry::discover_all()?;

    // Create semantic mapper
    let mapper = SemanticMapper::new(registry, 6); // max depth = 6

    // Generate strategy from genome
    let genome = vec![42, 17, 88, 3, 45, 12, 99, 234, 156, 78];
    let ast = mapper.create_strategy_ast(&genome)?;

    println!("Generated AST: {:#?}", ast);

    // Validate
    let validator = LightweightValidator::new(registry.clone(), 10);
    validator.validate(&ast)?;

    println!("AST is valid!");

    Ok(())
}
```

## Verification

### Test 1: Deterministic Generation
```rust
#[test]
fn test_deterministic_generation() {
    let registry = create_test_registry();
    let mapper = SemanticMapper::new(registry, 5);

    let genome = vec![42, 17, 88, 3, 45];

    let ast1 = mapper.create_strategy_ast(&genome).unwrap();
    let ast2 = mapper.create_strategy_ast(&genome).unwrap();

    // Same genome should produce identical AST
    assert_eq!(ast1, ast2);
}
```

### Test 2: Type Validity
```rust
#[test]
fn test_type_validity() {
    let registry = create_test_registry();
    let mapper = SemanticMapper::new(registry.clone(), 5);

    // Generate 100 random strategies
    for _ in 0..100 {
        let genome: Vec<u32> = (0..50).map(|_| rand::random()).collect();
        let ast = mapper.create_strategy_ast(&genome).unwrap();

        // All should pass validation
        let validator = LightweightValidator::new(registry.clone(), 10);
        assert!(validator.validate(&ast).is_ok());
    }
}
```

### Test 3: Smart Parameter Generation
```rust
#[test]
fn test_smart_parameters() {
    let registry = create_test_registry();
    let mapper = SemanticMapper::new(registry, 5);

    let genome = vec![42, 17, 88]; // Will generate RSI
    let ast = mapper.create_strategy_ast(&genome).unwrap();

    // Extract RSI period parameter
    let period = extract_rsi_period(&ast);

    // Should be from typical periods: [9, 14, 21, 25]
    assert!([9, 14, 21, 25].contains(&period));
}
```

## Common Issues

### Issue: Generated strategies are too simple
**Solution**: Increase `genome_length` in evolution config. Longer genomes allow more complex strategies.

### Issue: All strategies look the same
**Solution**: Increase `gene_range` to provide more variation in function choices.

### Issue: Type errors during execution
**Solution**: Check that all indicators are properly registered with correct input/output types.

## Next Steps

Proceed to **[13-optimization-methods.md](./13-optimization-methods.md)** to implement Walk-Forward Optimization and data splitting strategies.
