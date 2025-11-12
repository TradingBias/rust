# Phase 1: Critical Compilation Fixes

**Goal**: Resolve all 50+ compilation errors and achieve a clean `cargo build`

**Priority**: CRITICAL - Blocks all other work

## Task Breakdown

### 1. Quick Wins - Import & Capitalization Fixes (15 changes)

#### 1.1 Fix TradeBiasError Capitalization
**Files affected**: 6 files
**Change**: Replace `TradeBiasError` with `TradebiasError` (lowercase 'b')

```rust
// In each file, change:
use crate::error::TradeBiasError;
// To:
use crate::error::TradebiasError;
```

**Files**:
- `src/engines/generation/evolution_engine.rs:8`
- `src/engines/generation/optimisation/splitters/base.rs:3`
- `src/engines/generation/optimisation/splitters/simple.rs:3`
- `src/engines/generation/optimisation/splitters/wfo.rs:4`
- `src/engines/generation/optimisation/methods/wfo.rs:9`
- `src/engines/generation/lightweight_validator.rs` (check for TradeBiasError)

#### 1.2 Remove ast_converter Module Reference
**File**: `src/utils/mod.rs:2`
**Change**: Comment out or remove the line

```rust
// Remove or comment out:
// pub mod ast_converter;
```

#### 1.3 Add Missing rand::Rng Import
**File**: `src/engines/generation/evolution_engine.rs`
**Change**: Add trait import at top of file

```rust
// Add to imports section:
use rand::Rng;
```

#### 1.4 Fix ASTNode Capitalization
**File**: `src/engines/generation/diversity_validator.rs:37`
**Change**: Replace `ASTNode` with `AstNode`

```rust
// Change:
node: &ASTNode,
// To:
node: &AstNode,
```

And add import:
```rust
use crate::types::AstNode;
```

#### 1.5 Add Missing StrategyAST Import
**File**: `src/engines/generation/evolution_engine.rs`
**Change**: Add import at top

```rust
use crate::engines::generation::ast::StrategyAST;
```

#### 1.6 Add Missing AstNode Import
**File**: `src/engines/generation/lightweight_validator.rs`
**Change**: Add import at top

```rust
use crate::types::AstNode;
```

#### 1.7 Fix Genome Module Export
**File**: `src/engines/generation/mod.rs`
**Change**: Ensure genome is properly exposed

```rust
// Add/verify this line exists:
pub mod genome;
pub use genome::Genome;
```

### 2. Error Type Extensions (2 changes)

#### 2.1 Add Missing Error Variants
**File**: `src/error.rs`
**Change**: Add Generation and Validation variants to TradebiasError enum

```rust
#[derive(Debug, Clone)]
pub enum TradebiasError {
    // ... existing variants ...

    /// Error during strategy generation
    Generation(String),

    /// Error during validation
    Validation(String),

    /// Error during computation
    Computation(String),
}
```

And update Display/Error implementations:
```rust
impl std::fmt::Display for TradebiasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // ... existing matches ...
            TradebiasError::Generation(msg) => write!(f, "Generation error: {}", msg),
            TradebiasError::Validation(msg) => write!(f, "Validation error: {}", msg),
            TradebiasError::Computation(msg) => write!(f, "Computation error: {}", msg),
        }
    }
}
```

### 3. Polars API Compatibility (3 changes)

#### 3.1 Check Polars Version in Cargo.toml
**File**: `Cargo.toml`
**Investigation needed**: Check current Polars version

**Action**: Verify polars version and feature flags. The errors suggest methods like `abs()`, `diff()`, `cum_sum()` are missing.

Current errors occur in:
- `abs()` - used in primitives.rs, volatility.rs
- `diff()` - used in momentum.rs
- `cum_sum()` - used in volume.rs

**Expected fix**: Update to polars 0.36+ or add missing feature flags:

```toml
[dependencies]
polars = { version = "0.36", features = ["lazy", "temporal", "dtype-full", "rolling_window"] }
```

#### 3.2 Fix WindowType Usage
**Files**:
- `src/engines/generation/optimisation/splitters/simple.rs`
- `src/engines/generation/optimisation/splitters/wfo.rs`

**Change**: Polars' WindowType doesn't have Sliding/Anchored variants

Replace:
```rust
WindowType::Sliding(period)
WindowType::Anchored
```

With correct Polars API (check version-specific documentation):
```rust
// Use RollingOptionsFixedWindow instead
use polars::prelude::RollingOptionsFixedWindow;
```

### 4. Trait Architecture Redesign (CRITICAL - 1 major change)

#### 4.1 Fix StrategyFunction Trait Conflicts
**Problem**: Conflicting blanket implementations in `src/functions/traits.rs:99`

**Files**:
- `src/functions/traits.rs`
- `src/functions/strategy.rs`
- `src/functions/registry.rs`

**Current problematic code**:
```rust
// This causes conflicts:
impl<T: Indicator> StrategyFunction for T { ... }
impl<T: Primitive> StrategyFunction for T { ... }
```

**Solution**: Refactor to composition instead of inheritance

**Option A - Remove StrategyFunction trait entirely:**

```rust
// In src/functions/registry.rs
pub struct FunctionRegistry {
    indicators: HashMap<String, Arc<dyn Indicator>>,
    primitives: HashMap<String, Arc<dyn Primitive>>,
}

impl FunctionRegistry {
    pub fn get_indicator(&self, name: &str) -> Option<Arc<dyn Indicator>> {
        self.indicators.get(name).cloned()
    }

    pub fn get_primitive(&self, name: &str) -> Option<Arc<dyn Primitive>> {
        self.primitives.get(name).cloned()
    }

    pub fn get_function_by_name(&self, name: &str) -> Option<FunctionType> {
        if let Some(ind) = self.indicators.get(name) {
            return Some(FunctionType::Indicator(ind.clone()));
        }
        if let Some(prim) = self.primitives.get(name) {
            return Some(FunctionType::Primitive(prim.clone()));
        }
        None
    }
}

pub enum FunctionType {
    Indicator(Arc<dyn Indicator>),
    Primitive(Arc<dyn Primitive>),
}
```

**Option B - Use enum-based registry:**

```rust
// In src/functions/strategy.rs
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

// In src/functions/registry.rs
pub struct FunctionRegistry {
    functions: HashMap<String, StrategyFunction>,
}
```

**Recommendation**: Use Option B (enum-based) as it's cleaner and avoids trait conflicts entirely.

**Files to update**:
1. `src/functions/strategy.rs` - Replace trait with enum
2. `src/functions/traits.rs` - Remove blanket implementations (lines ~99)
3. `src/functions/registry.rs` - Update to use StrategyFunction enum
4. `src/engines/generation/semantic_mapper.rs` - Update method calls

### 5. AST Type System Resolution (CRITICAL - 1 major decision)

#### 5.1 Resolve StrategyAST vs AstNode Inconsistency

**Problem**: Code uses both `StrategyAST` and `AstNode` for representing strategies

**Files affected**:
- `src/engines/generation/evolution_engine.rs:143` - returns StrategyAST
- `src/engines/evaluation/backtester.rs` - expects &AstNode
- `src/engines/generation/ast.rs` - defines StrategyAST
- `src/types/mod.rs` - defines AstNode

**Investigation needed**: Determine the relationship between these types

**Option A - StrategyAST is wrapper around AstNode:**
```rust
// In src/engines/generation/ast.rs
pub struct StrategyAST {
    pub root: AstNode,
    pub metadata: StrategyMetadata,
}

impl StrategyAST {
    pub fn as_node(&self) -> &AstNode {
        &self.root
    }
}
```

Then update backtester call:
```rust
// In evolution_engine.rs:143
backtester.run(&strategy_ast.as_node(), &data)
```

**Option B - Use AstNode everywhere:**
Remove StrategyAST entirely and use AstNode with metadata stored separately.

**Recommendation**: Review the actual code to determine which approach is intended, then apply consistently across all files.

### 6. Semantic Mapper Type Fixes (1 change)

#### 6.1 Fix Type Mismatch in build_indicator_arguments
**File**: `src/engines/generation/semantic_mapper.rs:128`

**Problem**: Passes `&dyn Indicator` to function expecting `&dyn StrategyFunction`

**Solution**: Update after StrategyFunction trait is redesigned (depends on 4.1)

```rust
// After implementing enum-based StrategyFunction:
fn build_indicator_arguments(
    &self,
    indicator: &StrategyFunction,  // Changed parameter type
    required_args: usize,
    rng: &mut impl Rng,
) -> Vec<Box<AstNode>> {
    // Get indicator-specific info
    let ind = indicator.as_indicator()
        .expect("StrategyFunction must be Indicator variant");

    // ... rest of implementation
}
```

## Implementation Order

Execute in this sequence to minimize cascading errors:

1. **Quick wins first** (Tasks 1.1-1.7) - Reduces error count immediately
2. **Error variants** (Task 2.1) - Enables other code to compile
3. **Polars compatibility** (Tasks 3.1-3.2) - Critical for indicators
4. **Trait architecture** (Task 4.1) - Major refactor
5. **AST resolution** (Task 5.1) - Depends on understanding codebase intent
6. **Semantic mapper** (Task 6.1) - Depends on trait refactor

## Success Criteria

After Phase 1 completion:
- [ ] `cargo check` shows 0 errors
- [ ] All imports resolve correctly
- [ ] Trait architecture compiles without conflicts
- [ ] Polars methods are accessible
- [ ] AST types are used consistently

## Notes for AI Implementation

- **Do NOT run `cargo test`** - Only implement code changes
- **Focus on compilation** - Runtime correctness comes later
- **Document assumptions** - Especially for AST design decisions
- **Preserve existing logic** - Only fix types/imports, don't change algorithms
