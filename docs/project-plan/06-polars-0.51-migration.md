# Phase 0: Polars 0.51.0 Migration Plan

**Created**: November 12, 2025
**Status**: Ready for implementation
**Priority**: CRITICAL - Must be completed before Phase 1
**Blocking**: All other phases depend on this migration

## Overview

The project has successfully resolved the `raw_table_mut` dependency conflict by upgrading to Polars 0.51.0. However, this upgrade introduces 86 API compatibility errors that must be fixed before any other work can proceed.

**Dependency Status**: ✅ **SOLVED** - hashbrown conflict resolved
**Code Status**: ⚠️ **86 compilation errors** due to Polars API breaking changes
**Goal**: Update all project code to be compatible with Polars 0.51.0 API

## Error Breakdown

Total: **86 compilation errors** across the following categories:

| Category | Count | Error Type | Priority |
|----------|-------|------------|----------|
| Missing `output_type` trait | 30 | E0046 | High |
| Type mismatches | 15 | E0308 | High |
| Datetime accessor changes | 4 | E0599 | Medium |
| `RollingOptions` imports | 4 | E0432 | Medium |
| `abs()` method missing | 3 | E0599 | Medium |
| Capitalization errors | 3 | E0412 | Low |
| `LiteralValue::Int64` | 2 | E0599 | Medium |
| `cum_sum()` method missing | 2 | E0599 | Medium |
| Missing `window_type` field | 2 | E0063 | Medium |
| Other errors | 17 | Various | Low |

## Task Groups

### Task Group 1: High Priority API Changes (45 errors)

#### Task 1.1: Fix Indicator Trait - Add `output_type` Method (30 files)
**Error**: `error[E0046]: not all trait items implemented, missing: 'output_type'`
**Affected Files**: All indicator implementations (30 files)

**Root Cause**: The `Indicator` trait in Polars 0.51.0 now requires an `output_type` method that was optional or didn't exist in 0.41.

**Solution Pattern**:
```rust
// Add this method to every indicator implementation
// Note: Use types::DataType (from crate::types), NOT PolarsDataType
fn output_type(&self) -> types::DataType {
    types::DataType::Float64  // or appropriate type for the indicator
}
// Or if types::DataType is imported:
use crate::types::DataType;
fn output_type(&self) -> DataType {
    DataType::Float64
}
```

**Files to Fix**:
1. `src/indicators/ema.rs`
2. `src/indicators/sma.rs`
3. `src/indicators/rsi.rs`
4. `src/indicators/macd.rs`
5. `src/indicators/bollinger_bands.rs`
6. `src/indicators/stochastic.rs`
7. `src/indicators/atr.rs`
8. `src/indicators/adx.rs`
9. `src/indicators/cci.rs`
10. `src/indicators/mfi.rs`
11. `src/indicators/obv.rs`
12. `src/indicators/vwap.rs`
13. `src/indicators/momentum.rs`
14. `src/indicators/roc.rs`
15. `src/indicators/williams_r.rs`
16. `src/indicators/aroon.rs`
17. `src/indicators/keltner.rs`
18. `src/indicators/donchian.rs`
19. `src/indicators/ichimoku.rs`
20. `src/indicators/parabolic_sar.rs`
21. `src/functions/indicators/momentum.rs`
22. `src/functions/indicators/trend.rs`
23. `src/functions/indicators/volatility.rs`
24. `src/functions/indicators/volume.rs`
25-30. (Other indicator files as discovered during implementation)

**Implementation Steps**:
1. Search for all files implementing the `Indicator` trait
2. For each file, add the `output_type()` method
3. Choose appropriate return type based on indicator:
   - Most indicators: `PolarsDataType::Float64`
   - Boolean indicators (signals): `PolarsDataType::Boolean`
   - Integer indicators: `PolarsDataType::Int64`

**Verification**: Run `cargo build 2>&1 | grep "output_type"` should return 0 errors

---

#### Task 1.2: Fix Series::new() Signature - String to PlSmallStr (15 files)
**Error**: `error[E0308]: mismatched types - expected 'PlSmallStr', found '&str'`
**Affected Files**: Multiple files creating Series objects

**Root Cause**: In Polars 0.51.0, `Series::new()` first parameter changed from `&str` to `PlSmallStr`.

**Solution Patterns**:

```rust
// Pattern 1: String literal
// OLD (polars 0.41):
Series::new("column_name", data)

// NEW (polars 0.51):
Series::new("column_name".into(), data)

// Pattern 2: Formatted string
// OLD:
Series::new(&format!("return_{}", window), returns)

// NEW:
Series::new(format!("return_{}", window).into(), returns)
// OR
Series::new(&format!("return_{}", window).into(), returns)

// Pattern 3: Variable
// OLD:
let name = "column";
Series::new(name, data)

// NEW:
let name = "column";
Series::new(name.into(), data)
// OR
let name: PlSmallStr = "column".into();
Series::new(name, data)
```

**Files to Fix**:
1. `src/ml/features/engineer.rs` (multiple instances)
2. `src/ml/signals/extractor.rs`
3. `src/engines/evaluation/expression.rs`
4. `src/engines/generation/lightweight_validator.rs`
5. `src/indicators/*.rs` (various indicator files)
6-15. (Other files as discovered)

**Search Command**:
```bash
grep -rn "Series::new(" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "expected \`PlSmallStr\`"` should return 0 errors

---

### Task Group 2: Medium Priority API Changes (15 errors)

#### Task 2.1: Fix Datetime Accessors - Add `.phys` Prefix (4 files)
**Error**: `error[E0599]: no method named 'get' found for reference '&Logical<DatetimeType, Int64Type>'`

**Root Cause**: In Polars 0.51.0, datetime column accessors require going through the `.phys` field to access physical values.

**Solution Pattern**:
```rust
// OLD (polars 0.41):
if let Some(ts_ms) = timestamps.get(idx) {
    // use ts_ms
}

// NEW (polars 0.51):
if let Some(ts_ms) = timestamps.phys.get(idx) {
    // use ts_ms
}
```

**Files to Fix**:
1. `src/ml/features/engineer.rs` (lines ~274, ~286)
2. `src/ml/signals/extractor.rs` (datetime access)
3. `src/engines/evaluation/backtester.rs` (if any)
4. `src/data/processors/resample.rs` (if exists)

**Search Command**:
```bash
grep -rn "timestamps.get(" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "Logical<DatetimeType"` should return 0 errors

---

#### Task 2.2: Fix RollingOptions Imports (4 files)
**Error**: `error[E0432]: unresolved import 'polars::prelude::RollingOptions'`

**Root Cause**: `RollingOptions` has been renamed to `RollingOptionsFixedWindow` in Polars 0.51.0.

**Solution Pattern**:
```rust
// OLD (polars 0.41):
use polars::prelude::RollingOptions;

// NEW (polars 0.51):
use polars::prelude::RollingOptionsFixedWindow;

// In code:
// OLD:
fn rolling_params(&self) -> Option<RollingOptions> {
    Some(RollingOptions {
        window_size: Duration::new(self.period as i64),
        // ...
    })
}

// NEW:
fn rolling_params(&self) -> Option<RollingOptionsFixedWindow> {
    Some(RollingOptionsFixedWindow {
        window_size: self.period,
        // ...
    })
}
```

**Files to Fix**:
1. `src/functions/primitives.rs`
2. `src/functions/indicators/momentum.rs`
3. `src/functions/indicators/volatility.rs`
4. `src/functions/indicators/volume.rs`

**Additional Changes**:
- Update trait definitions in `src/functions/traits.rs` if they reference `RollingOptions`
- Update method signatures that return `Option<RollingOptions>`

**Search Commands**:
```bash
grep -rn "RollingOptions" src/ --include="*.rs"
grep -rn "rolling_params" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "RollingOptions"` should return 0 errors

---

#### Task 2.3: Fix Polars Expr Methods - `abs()` (3 files)
**Error**: `error[E0599]: no method named 'abs' found for enum 'polars::prelude::Expr'`

**Root Cause**: The `abs()` method has been removed from `Expr` in Polars 0.51.0 or moved to a different API.

**Investigation Needed**: Check Polars 0.51.0 docs for the new way to compute absolute values.

**Possible Solution Patterns**:
```rust
// OLD (polars 0.41):
expr.abs()

// NEW (polars 0.51) - Option 1: Use DSL function
use polars::prelude::*;
expr.apply(|s| s.abs(), GetOutput::same_type())

// NEW (polars 0.51) - Option 2: Use built-in if available
// Check if there's a new method or function in polars::functions
```

**Files to Fix**:
1. `src/functions/primitives.rs` (line ~102: `dsl::abs()`)
2-3. (Other files using `.abs()` on Expr)

**Search Command**:
```bash
grep -rn "\.abs()" src/ --include="*.rs"
grep -rn "dsl::abs" src/ --include="*.rs"
```

**Action Required**:
1. Check Polars 0.51.0 documentation for abs implementation
2. Update all usages accordingly
3. May need to implement custom abs using `.apply()` if no built-in available

**Verification**: Run `cargo build 2>&1 | grep "no method named \`abs\`"` should return 0 errors

---

#### Task 2.4: Fix `cum_sum()` Method (2 files)
**Error**: `error[E0599]: no method named 'cum_sum' found for enum 'polars::prelude::Expr'`

**Root Cause**: The `cum_sum()` method may have been renamed or moved in Polars 0.51.0.

**Investigation Needed**: Check Polars 0.51.0 docs for cumulative sum API.

**Possible Solution Patterns**:
```rust
// OLD (polars 0.41):
expr.cum_sum(false)

// NEW (polars 0.51) - Possible new name:
expr.cumsum(false)
// OR
expr.cumulative_sum()
// OR check if it's now cum_sum with different signature
```

**Files to Fix**:
1-2. (Files using `.cum_sum()`)

**Search Command**:
```bash
grep -rn "\.cum_sum(" src/ --include="*.rs"
```

**Action Required**:
1. Check Polars 0.51.0 documentation for cumulative sum
2. Update method name if changed
3. Update parameters if signature changed

**Verification**: Run `cargo build 2>&1 | grep "cum_sum"` should return 0 errors

---

#### Task 2.5: Fix `diff()` Method (1 file)
**Error**: `error[E0599]: no method named 'diff' found for enum 'polars::prelude::Expr'`

**Root Cause**: Similar to `abs()` and `cum_sum()`, the `diff()` method may have been changed.

**Possible Solution Pattern**:
```rust
// OLD (polars 0.41):
expr.diff(1, NullBehavior::Ignore)

// NEW (polars 0.51):
// Check Polars docs - may be:
expr.diff(1)  // if NullBehavior param removed
// OR
expr.diff(1, DiffOptions::default())  // if wrapped in options struct
```

**Search Command**:
```bash
grep -rn "\.diff(" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "no method named \`diff\`"` should return 0 errors

---

#### Task 2.6: Fix LiteralValue::Int64 (2 files)
**Error**: `error[E0599]: no variant or associated item named 'Int64' found for enum 'LiteralValue'`

**Root Cause**: `LiteralValue` enum variants may have been renamed or restructured in Polars 0.51.0.

**Investigation Needed**: Check Polars 0.51.0 `LiteralValue` enum definition.

**Possible Solution Pattern**:
```rust
// OLD (polars 0.41):
LiteralValue::Int64(value)

// NEW (polars 0.51) - Possible new variants:
LiteralValue::Int(value)  // unified integer type
// OR
LiteralValue::Integer(value)
// OR check if you should use lit() function instead
```

**Search Command**:
```bash
grep -rn "LiteralValue::Int64" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "LiteralValue"` should return 0 errors

---

#### Task 2.7: Fix SplitConfig Missing Field (2 files)
**Error**: `error[E0063]: missing field 'window_type' in initializer of 'splitters::types::SplitConfig'`

**Root Cause**: The `SplitConfig` struct now requires a `window_type` field that didn't exist before.

**Solution Pattern**:
```rust
// OLD (polars 0.41):
SplitConfig {
    train_size: 0.8,
    test_size: 0.2,
    // ...
}

// NEW (polars 0.51):
SplitConfig {
    train_size: 0.8,
    test_size: 0.2,
    window_type: WindowType::Fixed,  // or appropriate type
    // ...
}
```

**Files to Fix**:
1. `src/engines/generation/optimisation/splitters/*.rs`
2. (Other files constructing SplitConfig)

**Action Required**:
1. Check the definition of `SplitConfig` in `src/engines/generation/optimisation/splitters/types.rs`
2. Determine what `window_type` field expects
3. Add appropriate default value to all SplitConfig constructions

**Search Command**:
```bash
grep -rn "SplitConfig {" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "window_type"` should return 0 errors

---

### Task Group 3: Low Priority Fixes (26 errors)

#### Task 3.1: Fix Error Type Capitalization (3 files)
**Error**: `error[E0412]: cannot find type 'TradeBiasError' in this scope`

**Root Cause**: Inconsistent capitalization - the error enum is `TradebiasError` but some code references `TradeBiasError`.

**Solution Pattern**:
```rust
// WRONG:
Result<Vec<DataSplit>, TradeBiasError>

// CORRECT:
Result<Vec<DataSplit>, TradebiasError>
```

**Files to Fix**:
1. `src/engines/generation/optimisation/splitters/base.rs`
2. `src/engines/generation/optimisation/methods/base.rs`
3. `src/engines/generation/optimisation/methods/wfo.rs`

**Search Command**:
```bash
grep -rn "TradeBiasError" src/ --include="*.rs"
```

**Fix**: Replace all instances of `TradeBiasError` with `TradebiasError` (note lowercase 'b')

**Verification**: Run `cargo build 2>&1 | grep "TradeBiasError"` should return 0 errors

---

#### Task 3.2: Fix DataFrame::new() - Series to Column (1 file)
**Error**: `error[E0308]: mismatched types - expected 'Vec<Column>', found 'Vec<Series>'`

**Root Cause**: `DataFrame::new()` now expects `Vec<Column>` instead of `Vec<Series>` in Polars 0.51.0.

**Solution Pattern**:
```rust
// OLD (polars 0.41):
let series_vec: Vec<Series> = vec![...];
DataFrame::new(series_vec)

// NEW (polars 0.51):
let series_vec: Vec<Series> = vec![...];
let columns: Vec<Column> = series_vec.into_iter().map(Column::from).collect();
DataFrame::new(columns)

// OR if using Series directly:
DataFrame::new(vec![...].into_iter().map(Column::from).collect())
```

**File to Fix**:
1. `src/ml/features/engineer.rs` (line ~77)

**Search Command**:
```bash
grep -rn "DataFrame::new(" src/ --include="*.rs"
```

**Verification**: Run `cargo build 2>&1 | grep "Vec<Column>"` should return 0 errors

---

#### Task 3.3: Fix Cache Type Mismatch - Series vs Column (1 file)
**Error**: `error[E0308]: mismatched types - expected 'Series', found 'Column'`

**Root Cause**: Cache system expects `Series` but receives `Column` in Polars 0.51.0.

**Solution Pattern**:
```rust
// OLD (polars 0.41):
self.cache.set(cache_key, series.clone());

// NEW (polars 0.51):
// Option 1: Convert Column to Series
self.cache.set(cache_key, series.as_materialized_series().clone());

// Option 2: Change cache type to accept Column
// (requires modifying cache structure)
```

**File to Fix**:
1. `src/engines/evaluation/expression.rs` (line ~75)

**Action Required**:
1. Determine if cache should store `Series` or `Column`
2. Update cache type definition if needed
3. Add conversions where necessary

**Verification**: Run `cargo build 2>&1 | grep "cache.set"` should return 0 errors

---

#### Task 3.4: Fix PlSmallStr Reference Issue (1 file)
**Error**: `error[E0277]: the trait bound '&&PlSmallStr: Into<PlSmallStr>' is not satisfied`

**Root Cause**: Trying to pass a double reference `&&PlSmallStr` where `PlSmallStr` is expected.

**Solution Pattern**:
```rust
// Problem: Double reference
let name: &&PlSmallStr = ...;
Series::new(name.into(), data)  // FAILS

// Solution: Dereference or clone
Series::new((*name).clone(), data)  // Option 1
// OR
Series::new((**name).into(), data)  // Option 2
```

**File to Fix**: (Locate file with this error)

**Search Command**:
```bash
cargo build 2>&1 | grep -B 5 "&&PlSmallStr"
```

**Verification**: Run `cargo build 2>&1 | grep "&&PlSmallStr"` should return 0 errors

---

#### Task 3.5: Fix Trait Bound Issues (6 files)
**Errors**: Various `error[E0277]: the trait bound ... is not satisfied`

These include:
- `std::io::Error: Clone`
- `serde_json::Error: Clone`
- `primitives::Or: functions::traits::Primitive`
- `primitives::And: functions::traits::Primitive`
- `Abs: functions::traits::Primitive`
- `polars::prelude::Column: polars::prelude::Literal`

**Root Cause**: Multiple causes - some errors from wrapping types, some from trait implementations.

**Solution Approach**:
1. For Clone errors on std::io::Error and serde_json::Error:
   - Don't derive Clone on wrapper types containing these errors
   - Use Arc<> if sharing is needed
   - Implement custom Clone that maps errors

2. For Primitive trait bound errors:
   - Implement the `Primitive` trait for `Or`, `And`, and `Abs` types
   - Or restructure code to not require these as Primitives

3. For Column/Literal trait bound:
   - Convert Column to Series where Literal trait is needed
   - Or use appropriate Polars API for literals

**Action Required**: Address each trait bound error individually based on context.

---

#### Task 3.6: Fix Module and Import Errors (2 files)
**Errors**:
- `error[E0583]: file not found for module 'genome'`
- `error[E0432]: unresolved import 'crate::utils::ast_converter'`

**Solution**:
1. `genome` module:
   - Create `src/engines/generation/genome.rs` if missing
   - Or remove `pub mod genome;` from `src/engines/generation/mod.rs` if not needed

2. `ast_converter` import:
   - Create `src/utils/ast_converter.rs` if missing
   - Or update import path if module exists elsewhere
   - Or remove import if not needed

**Verification**: Run `cargo build 2>&1 | grep -E "E0583|E0432"` should return 0 errors

---

#### Task 3.7: Fix StrategyAST Import/Type Errors (2 files)
**Errors**:
- `error[E0412]: cannot find type 'StrategyAST' in this scope`
- `error[E0433]: failed to resolve: use of undeclared type 'StrategyAST'`

**Solution**:
```rust
// Add import at top of file
use crate::engines::generation::StrategyAST;

// Or use fully qualified path
crate::engines::generation::StrategyAST
```

**Files to Fix**:
1. `src/engines/evaluation/backtester.rs`
2. (Other files using StrategyAST)

**Verification**: Run `cargo build 2>&1 | grep "StrategyAST"` should return 0 errors

---

#### Task 3.8: Fix Borrow Checker and Ownership Issues (1 file)
**Error**: `error[E0382]: borrow of partially moved value: 'func'`

**File**: `src/engines/generation/lightweight_validator.rs`

**Solution**: Requires examining specific code context and either:
- Clone the value before moving
- Restructure code to avoid partial move
- Use references instead of owned values

---

#### Task 3.9: Fix Method Call Issues (3 files)
**Errors**:
- `error[E0609]: no field 'try_' on type 'HallOfFame'`
- `error[E0599]: the method 'call' exists ... but its trait bounds were not satisfied`
- `error[E0599]: no method named 'name' found for reference '&dyn Indicator'`

**Solutions**:
1. `try_` field: Check if field was renamed or removed in struct definition
2. `call` method trait bounds: Ensure proper trait imports and implementations
3. `name` method: Add method to trait definition or use alternative API

---

#### Task 3.10: Fix Ambiguous Associated Type (2 files)
**Error**: `error[E0223]: ambiguous associated type`

**Solution**: Use fully qualified syntax
```rust
// Instead of:
Type::AssociatedType

// Use:
<Type as Trait>::AssociatedType
```

---

#### Task 3.11: Fix Iterator Type Mismatch (1 file)
**Error**: `error[E0277]: a value of type 'Vec<Arc<dyn Indicator>>' cannot be built from an iterator over elements of type 'Arc<&dyn Indicator>'`

**Solution**:
```rust
// Problem:
let vec: Vec<Arc<dyn Indicator>> = iter.map(|x| Arc::new(x)).collect();

// Solution - clone the trait object properly:
let vec: Vec<Arc<dyn Indicator>> = iter.map(|x| Arc::new((**x).clone())).collect();
// OR restructure to avoid double references
```

---

## Implementation Order

Execute tasks in this order to minimize cascading errors:

### **Stage 1: Core API Changes** (Estimated: 2-3 hours)
1. Task 1.1 - Add `output_type` to all indicators (30 files)
2. Task 1.2 - Fix `Series::new()` signatures (15 files)
3. Task 2.2 - Fix `RollingOptions` imports (4 files)

**Checkpoint**: Run `cargo build` - should reduce errors from 86 to ~40

### **Stage 2: Method Changes** (Estimated: 1-2 hours)
4. Task 2.1 - Fix datetime accessors (4 files)
5. Task 2.3 - Fix `abs()` method (3 files)
6. Task 2.4 - Fix `cum_sum()` method (2 files)
7. Task 2.5 - Fix `diff()` method (1 file)
8. Task 2.6 - Fix `LiteralValue::Int64` (2 files)

**Checkpoint**: Run `cargo build` - should reduce errors from ~40 to ~20

### **Stage 3: Structural Changes** (Estimated: 1-2 hours)
9. Task 2.7 - Add `window_type` field (2 files)
10. Task 3.2 - Fix `DataFrame::new()` (1 file)
11. Task 3.3 - Fix cache type mismatch (1 file)
12. Task 3.6 - Fix module errors (2 files)
13. Task 3.7 - Fix StrategyAST imports (2 files)

**Checkpoint**: Run `cargo build` - should reduce errors from ~20 to ~10

### **Stage 4: Cleanup** (Estimated: 1 hour)
14. Task 3.1 - Fix capitalization (3 files)
15. Task 3.4 - Fix PlSmallStr reference (1 file)
16. Task 3.5 - Fix trait bounds (6 files)
17. Task 3.8 - Fix borrow checker issues (1 file)
18. Task 3.9 - Fix method call issues (3 files)
19. Task 3.10 - Fix ambiguous types (2 files)
20. Task 3.11 - Fix iterator type (1 file)

**Final Checkpoint**: Run `cargo build` - should compile successfully ✅

---

## Verification Strategy

After completing each stage:

1. **Run cargo build**:
   ```bash
   cargo build 2>&1 | tee build_output.txt
   ```

2. **Count remaining errors**:
   ```bash
   grep "error\[E" build_output.txt | wc -l
   ```

3. **Check progress**:
   - Stage 1 complete: ~40 errors remaining (53% fixed)
   - Stage 2 complete: ~20 errors remaining (77% fixed)
   - Stage 3 complete: ~10 errors remaining (88% fixed)
   - Stage 4 complete: 0 errors remaining (100% fixed) ✅

4. **Final verification**:
   ```bash
   cargo build --release
   cargo check
   ```

---

## Success Criteria

- [ ] All 86 compilation errors resolved
- [ ] `cargo build` completes without errors
- [ ] `cargo check` passes
- [ ] All Polars 0.51.0 API changes properly implemented
- [ ] No warnings related to Polars API usage
- [ ] Ready to proceed with Phase 1 of main project plan

---

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting `.into()` on String Literals
**Problem**: `Series::new("name", data)` still causes error
**Solution**: Always use `.into()`: `Series::new("name".into(), data)`

### Pitfall 2: Incorrect Datetime Access
**Problem**: `timestamps.get(idx)` doesn't work
**Solution**: Use `.phys` first: `timestamps.phys.get(idx)`

### Pitfall 3: Wrong RollingOptions Type
**Problem**: `RollingOptions` import fails
**Solution**: Use `RollingOptionsFixedWindow` instead

### Pitfall 4: Missing output_type Implementation
**Problem**: Indicator trait implementation incomplete
**Solution**: Add `fn output_type(&self) -> PolarsDataType { PolarsDataType::Float64 }`

### Pitfall 5: Series vs Column Confusion
**Problem**: Type mismatch with DataFrame::new()
**Solution**: Convert Series to Column: `.into_iter().map(Column::from).collect()`

---

## References

- **Polars Migration Guide**: https://docs.pola.rs/migration/
- **Polars 0.51.0 Release Notes**: https://github.com/pola-rs/polars/releases/tag/rs-0.51.0
- **PlSmallStr Documentation**: https://docs.rs/polars/latest/polars/prelude/struct.PlSmallStr.html
- **RollingOptionsFixedWindow**: https://docs.rs/polars/latest/polars/prelude/struct.RollingOptionsFixedWindow.html
- **semantic-mapper.md**: Full investigation of hashbrown/polars issue

---

## Status Tracking

Track progress by marking tasks as you complete them:

- [ ] **Stage 1: Core API Changes** (45 errors)
  - [ ] Task 1.1 - output_type implementations (30 errors)
  - [ ] Task 1.2 - Series::new() fixes (15 errors)
  - [ ] Task 2.2 - RollingOptions imports (4 errors)

- [ ] **Stage 2: Method Changes** (15 errors)
  - [ ] Task 2.1 - Datetime accessors (4 errors)
  - [ ] Task 2.3 - abs() method (3 errors)
  - [ ] Task 2.4 - cum_sum() method (2 errors)
  - [ ] Task 2.5 - diff() method (1 error)
  - [ ] Task 2.6 - LiteralValue::Int64 (2 errors)

- [ ] **Stage 3: Structural Changes** (10 errors)
  - [ ] Task 2.7 - window_type field (2 errors)
  - [ ] Task 3.2 - DataFrame::new() (1 error)
  - [ ] Task 3.3 - Cache type (1 error)
  - [ ] Task 3.6 - Module errors (2 errors)
  - [ ] Task 3.7 - StrategyAST imports (2 errors)

- [ ] **Stage 4: Cleanup** (16 errors)
  - [ ] Task 3.1 - Capitalization (3 errors)
  - [ ] Task 3.4 - PlSmallStr reference (1 error)
  - [ ] Task 3.5 - Trait bounds (6 errors)
  - [ ] Task 3.8 - Borrow checker (1 error)
  - [ ] Task 3.9 - Method calls (3 errors)
  - [ ] Task 3.10 - Ambiguous types (2 errors)
  - [ ] Task 3.11 - Iterator type (1 error)

**Final Status**: ⚠️ 86 errors remaining (No tasks implemented yet) → ✅ 0 errors (target)

---

**Last Updated**: 2025-11-12
**Plan Version**: 1.0
**Next Review**: After Stage 1 completion
