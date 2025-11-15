# Implementation Guide for AI Agents

This guide provides instructions for AI agents implementing the TradeBias AI project plan.

## Overview

The project plan is divided into 3 phases with ~140 specific tasks. This guide explains how to execute the plan systematically.

## Important Constraints

### ❌ DO NOT:
- Run `cargo test` or `cargo build` during implementation
- Debug code - only implement the specified changes
- Deviate from the specified implementations
- Skip tasks or reorder phases without justification
- Add extra features not in the plan

### ✅ DO:
- Follow the implementation order strictly
- Implement code changes exactly as specified
- Document any assumptions made
- Preserve existing algorithm logic when fixing types
- Keep changes minimal and focused

## Phase Execution Strategy

### Phase 1: Critical Compilation Fixes

**Goal**: Get `cargo build` to succeed

**Strategy**:
1. Start with "Quick Wins" (Tasks 1.1-1.7) - these are simple find/replace operations
2. Move to error variants (Task 2.1) - adds missing enum variants
3. Handle Polars compatibility (Tasks 3.1-3.2) - may require research
4. Tackle trait architecture (Task 4.1) - most complex, requires careful refactoring
5. Resolve AST issues (Task 5.1) - requires understanding code intent
6. Fix semantic mapper (Task 6.1) - depends on previous fixes

**Key Decision Points**:
- **Polars Version**: Check actual version, update if needed, verify API compatibility
- **StrategyFunction Design**: Choose enum-based approach (recommended) vs trait removal
- **AST Representation**: Determine if StrategyAST wraps AstNode or should be eliminated

**Files to Read First**:
- `src/functions/traits.rs` - understand current trait hierarchy
- `src/functions/registry.rs` - understand registry design
- `src/engines/generation/ast.rs` - understand StrategyAST
- `src/types/mod.rs` - understand AstNode
- `Cargo.toml` - check Polars version

### Phase 2: Core Features Implementation

**Goal**: Complete all partially implemented features

**Strategy**:
1. **Configuration System** (highest priority):
   - Create all config files in order: traits → sections → manager → mod.rs
   - Follow the template structure provided exactly
   - Don't implement `to_manifest()` fully if it's complex - use `todo!()`

2. **Polars DSL Fixes**:
   - After Phase 1 Polars update, verify all method calls
   - Check Polars documentation for current API
   - Use alternatives if methods don't exist

3. **Verify Existing Implementations**:
   - Check robustness validation compiles
   - Check semantic mapper works with new traits

**Key Decision Points**:
- **Config Manifest Generation**: Can be stubbed with `todo!()` if complex
- **Polars Methods**: Use current API, document alternatives used

**Files to Create**:
- All `src/config/*.rs` files as specified
- Update `src/error.rs` with Configuration variant

### Phase 3: Additional Features

**Goal**: Achieve 100% feature completeness

**Strategy**:
1. **Tier 2 Indicators**:
   - Implement in order: Momentum → Trend → Volatility → Volume
   - Use vectorized pattern for all except SAR (stateful)
   - Copy pattern from Tier 1 indicators
   - Register all in FunctionRegistry after implementation

2. **Data Connectors**:
   - Start with base trait
   - Implement CSV connector fully
   - Stub MT5 connector for future work

3. **Signal/Filter Engines**:
   - Create base traits first
   - Implement one example of each (meta-labeling, volatility filter)
   - Can stub complex logic with `todo!()`

**Key Decision Points**:
- **Indicator Complexity**: Some indicators (SAR) are stateful - follow spec carefully
- **MT5 Connector**: Stub is acceptable, full implementation is optional
- **Signal Engine Logic**: Can be stubbed if ML model integration is unclear

**Files to Reference**:
- `docs/ai-implementation/05-indicators-tier2.md` - indicator specs
- Existing Tier 1 indicators - pattern to follow
- `docs/ai-implementation/18-data-connectors.md` - connector specs

## Handling Ambiguity

When encountering ambiguous situations:

1. **Check Specifications**: Review relevant `docs/ai-implementation/*.md` file
2. **Check Existing Code**: Look for similar patterns in codebase
3. **Make Reasonable Choice**: Document assumption in code comment
4. **Use `todo!()`**: For complex logic that's not fully specified

Example:
```rust
// ASSUMPTION: Using enum-based StrategyFunction to avoid trait conflicts
// as per Rust orphan rules. Alternative would be separate registries.
pub enum StrategyFunction {
    Indicator(Arc<dyn Indicator>),
    Primitive(Arc<dyn Primitive>),
}
```

## Code Style Guidelines

### Import Organization
```rust
// Standard library
use std::collections::HashMap;
use std::sync::Arc;

// External crates
use polars::prelude::*;
use serde::{Deserialize, Serialize};

// Internal crates
use crate::error::TradebiasError;
use crate::types::AstNode;
```

### Error Handling
```rust
// Use ? operator for Results
let data = fetch_data()?;

// Provide context in error messages
.map_err(|e| TradebiasError::Data(format!("Failed to load CSV: {}", e)))?
```

### Documentation
```rust
/// Generates trading signals from ML model predictions
///
/// # Arguments
/// * `predictions` - DataFrame with model outputs
/// * `threshold` - Minimum confidence threshold
///
/// # Returns
/// DataFrame with binary signals (1=long, -1=short, 0=neutral)
pub fn generate_signals(
    predictions: &DataFrame,
    threshold: f64,
) -> Result<DataFrame, TradebiasError> {
    // Implementation
}
```

## Verification Strategy

After each phase, verify:

### Phase 1 Verification
```bash
# Should complete without errors
cargo check
```

### Phase 2 Verification
```bash
# Should still compile
cargo check

# Should show new config module
cargo tree | grep config
```

### Phase 3 Verification
```bash
# Should still compile with all features
cargo check

# Should show all indicators registered
# Check in code: FunctionRegistry::new().with_tier2_indicators()
```

## Common Pitfalls

### Pitfall 1: Trait Architecture
**Problem**: Trying to implement StrategyFunction as a trait leads to conflicts
**Solution**: Use enum-based approach as specified in Phase 1, Task 4.1

### Pitfall 2: Polars API Changes
**Problem**: Methods like `abs()` or `diff()` don't exist
**Solution**: Check Polars version, update if needed, use alternative APIs

### Pitfall 3: AST Type Confusion
**Problem**: Mixing StrategyAST and AstNode inconsistently
**Solution**: Decide on one approach and apply consistently (Phase 1, Task 5.1)

### Pitfall 4: Missing Imports
**Problem**: Types not found in scope
**Solution**: Add all necessary `use` statements as specified in Phase 1

### Pitfall 5: Indicator Complexity
**Problem**: Some Tier 2 indicators are complex
**Solution**: Follow the patterns from Tier 1, use `todo!()` for very complex logic

## Progress Tracking

Use this format to track progress:

```markdown
## Phase 1 Progress: 15/35 (43%)
- [x] 1.1.1 Fix TradeBiasError in evolution_engine.rs
- [x] 1.1.2 Fix TradeBiasError in splitters/base.rs
- [ ] 1.1.3 Fix TradeBiasError in splitters/simple.rs
...
```

## File Creation Template

When creating new files, use this template:

```rust
//! Brief module description
//!
//! Detailed explanation of what this module does

use crate::error::TradebiasError;
// ... other imports

/// Main struct/trait documentation
pub struct MyStruct {
    // fields
}

impl MyStruct {
    /// Constructor documentation
    pub fn new() -> Self {
        // implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // TODO: Add test after Phase 3
    }
}
```

## Final Checklist

Before marking any phase complete:

### Phase 1 Complete When:
- [ ] All import errors resolved
- [ ] All capitalization fixed
- [ ] Trait architecture redesigned
- [ ] AST types consistent
- [ ] Polars API compatible
- [ ] `cargo check` succeeds

### Phase 2 Complete When:
- [ ] Configuration system fully implemented
- [ ] All config files created
- [ ] Polars DSL issues fixed
- [ ] Existing partial implementations verified
- [ ] `cargo check` still succeeds

### Phase 3 Complete When:
- [ ] All 20 Tier 2 indicators implemented
- [ ] All indicators registered
- [ ] Data connector framework complete
- [ ] CSV connector functional
- [ ] Signal/filter framework implemented
- [ ] `cargo check` still succeeds
- [ ] All items in `implementation_summary.md` can be marked "Implemented"

## Success Criteria

Project is complete when:
1. ✅ `cargo build` succeeds without errors or warnings
2. ✅ All features from `docs/ai-implementation/*.md` implemented
3. ✅ All tasks in all 3 phases marked complete
4. ✅ Code follows Rust best practices
5. ✅ Documentation is adequate

## Next Steps After Implementation

After AI completes all phases:
1. Human review of code quality
2. Run `cargo test` to verify functionality
3. Fix any test failures
4. Add integration tests
5. Performance profiling and optimization
6. Deploy to production

---

**Remember**: Focus on code changes only. No testing, no debugging, just implementation. The human will handle verification and testing after completion.
