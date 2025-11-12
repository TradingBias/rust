# Quick Reference Checklist

This document provides a condensed checklist of all tasks across all phases.

## Phase 1: Critical Compilation Fixes (35 tasks)

### Import & Capitalization (15 tasks)
- [x] 1.1.1 Fix `TradeBiasError` → `TradebiasError` in `evolution_engine.rs:8`
- [x] 1.1.2 Fix `TradeBiasError` → `TradebiasError` in `optimisation/splitters/base.rs:3`
- [x] 1.1.3 Fix `TradeBiasError` → `TradebiasError` in `optimisation/splitters/simple.rs:3`
- [x] 1.1.4 Fix `TradeBiasError` → `TradebiasError` in `optimisation/splitters/wfo.rs:4`
- [x] 1.1.5 Fix `TradeBiasError` → `TradebiasError` in `optimisation/methods/wfo.rs:9`
- [x] 1.1.6 Fix `TradeBiasError` → `TradebiasError` in `lightweight_validator.rs`
- [x] 1.2 Remove `ast_converter` module from `src/utils/mod.rs:2`
- [x] 1.3 Add `use rand::Rng;` to `evolution_engine.rs`
- [x] 1.4.1 Fix `ASTNode` → `AstNode` in `diversity_validator.rs:37`
- [x] 1.4.2 Add `use crate::types::AstNode;` to `diversity_validator.rs`
- [x] 1.5 Add `use crate::engines::generation::ast::StrategyAST;` to `evolution_engine.rs`
- [x] 1.6 Add `use crate::types::AstNode;` to `lightweight_validator.rs`
- [x] 1.7.1 Add `pub mod genome;` to `src/engines/generation/mod.rs`
- [x] 1.7.2 Add `pub use genome::Genome;` to `src/engines/generation/mod.rs`

### Error Type Extensions (2 tasks)
- [x] 2.1.1 Add `Generation(String)` variant to `TradebiasError` in `src/error.rs`
- [x] 2.1.2 Add `Validation(String)` variant to `TradebiasError` in `src/error.rs`
- [x] 2.1.3 Add `Computation(String)` variant to `TradebiasError` in `src/error.rs`
- [x] 2.1.4 Update `Display` implementation for new variants
- [x] 2.1.5 Update `Error` implementation if needed

### Polars API Compatibility (5 tasks)
- [x] 3.1.1 Check Polars version in `Cargo.toml`
- [x] 3.1.2 Update Polars to version 0.36+ if needed
- [x] 3.1.3 Add required Polars feature flags
- [x] 3.2.1 Fix `WindowType` usage in `optimisation/splitters/simple.rs`
- [x] 3.2.2 Fix `WindowType` usage in `optimisation/splitters/wfo.rs`

### Trait Architecture Redesign (8 tasks)
- [x] 4.1.1 Create `StrategyFunction` enum in `src/functions/strategy.rs`
- [x] 4.1.2 Implement helper methods on `StrategyFunction` enum
- [x] 4.1.3 Remove blanket trait implementations from `src/functions/traits.rs`
- [x] 4.1.4 Update `FunctionRegistry` to use enum in `src/functions/registry.rs`
- [x] 4.1.5 Update `FunctionRegistry::get_function_by_name()`
- [x] 4.1.6 Update `FunctionRegistry::register_indicator()`
- [x] 4.1.7 Update `FunctionRegistry::register_primitive()`
- [x] 4.1.8 Update `semantic_mapper.rs` to use new registry API

### AST Type System Resolution (3 tasks)
- [x] 5.1.1 Determine relationship between `StrategyAST` and `AstNode`
- [x] 5.1.2 Implement `as_node()` method on `StrategyAST` OR remove `StrategyAST`
- [x] 5.1.3 Update all backtester calls to use consistent type

### Semantic Mapper Fixes (2 tasks)
- [ ] 6.1.1 Update `build_indicator_arguments` parameter types
- [ ] 6.1.2 Update method calls to work with enum-based registry

**BLOCKER:** The project does not compile due to an error in the `polars-core` dependency: `no method named 'raw_table_mut' found for struct 'HashMap'`. This prevents further progress.

## Phase 2: Core Features Implementation (45+ tasks)

### Configuration System (25 tasks)
- [ ] 1.1 Create `src/config/traits.rs` with `ConfigSection` trait
- [ ] 1.2.1 Create `src/config/evolution.rs`
- [ ] 1.2.2 Implement `EvolutionConfig` struct
- [ ] 1.2.3 Implement `Default` for `EvolutionConfig`
- [ ] 1.2.4 Implement `ConfigSection` for `EvolutionConfig`
- [ ] 1.3.1 Create `src/config/backtesting.rs`
- [ ] 1.3.2 Implement `BacktestingConfig` struct
- [ ] 1.3.3 Implement `Default` for `BacktestingConfig`
- [ ] 1.3.4 Implement `ConfigSection` for `BacktestingConfig`
- [ ] 1.4.1 Create `src/config/trade_management.rs`
- [ ] 1.4.2 Implement `TradeManagementConfig` struct
- [ ] 1.4.3 Implement `Default` for `TradeManagementConfig`
- [ ] 1.4.4 Implement `ConfigSection` for `TradeManagementConfig`
- [ ] 1.5.1 Create `src/config/ml.rs`
- [ ] 1.5.2 Implement `MLConfig` struct
- [ ] 1.5.3 Implement `Default` for `MLConfig`
- [ ] 1.5.4 Implement `ConfigSection` for `MLConfig`
- [ ] 1.6.1 Create `src/config/manager.rs`
- [ ] 1.6.2 Implement `AppConfig` struct
- [ ] 1.6.3 Implement `ConfigManager` struct
- [ ] 1.6.4 Implement `load_from_file()` method
- [ ] 1.6.5 Implement `save_to_file()` method
- [ ] 1.7 Update `src/config/mod.rs` with module exports
- [ ] 1.8 Add `Configuration(String)` variant to `TradebiasError`
- [ ] 1.9 Add `toml = "0.8"` to `Cargo.toml`

### Polars DSL Fixes (6 tasks)
- [ ] 2.1.1 Review Polars method calls in `src/functions/primitives.rs`
- [ ] 2.1.2 Fix `abs()` usage if broken
- [ ] 3.1.1 Fix `diff()` method in `src/functions/momentum.rs`
- [ ] 3.1.2 Verify alternative implementation if needed
- [ ] 3.2.1 Fix `cum_sum()` → `cumsum()` in `src/functions/volume.rs`
- [ ] 3.2.2 Test volume indicators compile

### Robustness Validation (5 tasks)
- [ ] 4.1.1 Verify `src/engines/validation/robustness/base.rs` compiles
- [ ] 4.1.2 Verify `monte_carlo.rs` compiles
- [ ] 4.1.3 Verify `parameter_stability.rs` compiles
- [ ] 4.1.4 Verify `friction.rs` compiles
- [ ] 4.1.5 Verify `orchestrator.rs` compiles

### Semantic Generation (3 tasks)
- [ ] 5.1.1 Verify `semantic_mapper.rs` compiles after trait refactor
- [ ] 5.1.2 Check `FunctionRegistry` usage is correct
- [ ] 5.1.3 Verify AST building works properly

## Phase 3: Additional Features (60+ tasks)

### Tier 2 Indicators - Momentum (7 tasks)
- [ ] 1.1.1 Implement `WilliamsR` indicator
- [ ] 1.1.2 Implement `MFI` indicator
- [ ] 1.1.3 Implement `ROC` indicator
- [ ] 1.1.4 Implement `DeMarker` indicator
- [ ] 1.1.5 Implement `RVI` indicator
- [ ] 1.1.6 Implement `Force` indicator
- [ ] 1.1.7 Implement `TriX` indicator

### Tier 2 Indicators - Trend (6 tasks)
- [ ] 1.2.1 Implement `DEMA` indicator
- [ ] 1.2.2 Implement `TEMA` indicator
- [ ] 1.2.3 Implement `Envelopes` indicator
- [ ] 1.2.4 Implement `SAR` indicator (stateful)
- [ ] 1.2.5 Implement `Bulls` indicator
- [ ] 1.2.6 Implement `Bears` indicator

### Tier 2 Indicators - Volatility (2 tasks)
- [ ] 1.3.1 Implement `StdDev` indicator wrapper
- [ ] 1.3.2 Implement `Chaikin` volatility indicator

### Tier 2 Indicators - Volume (5 tasks)
- [ ] 1.4.1 Implement `Volumes` indicator
- [ ] 1.4.2 Implement `BWMFI` indicator
- [ ] 1.4.3 Implement `AC` indicator
- [ ] 1.4.4 Implement `AO` indicator
- [ ] 1.4.5 Implement `VolumeMomentum` indicator

### Tier 2 Registration (1 task)
- [ ] 1.5 Register all 20 Tier 2 indicators in `FunctionRegistry`

### Data Connectors (12 tasks)
- [ ] 2.1.1 Create `src/data/connectors/base.rs`
- [ ] 2.1.2 Implement `DataConnector` trait
- [ ] 2.1.3 Implement `Timeframe` enum
- [ ] 2.2.1 Create `src/data/connectors/csv.rs`
- [ ] 2.2.2 Implement `CsvConnector` struct
- [ ] 2.2.3 Implement `fetch_ohlcv()` for CSV
- [ ] 2.2.4 Implement `health_check()` for CSV
- [ ] 2.3.1 Create `src/data/connectors/mt5.rs`
- [ ] 2.3.2 Implement `MT5Connector` stub
- [ ] 2.4 Update `src/data/mod.rs`
- [ ] 2.5 Add `Data(String)` variant to `TradebiasError`
- [ ] 2.6 Add `async-trait` and `tokio` to `Cargo.toml`

### Signal & Filtering Engines (10 tasks)
- [ ] 3.1.1 Create `src/ml/signals/base.rs`
- [ ] 3.1.2 Implement `SignalGenerator` trait
- [ ] 3.2.1 Create `src/ml/signals/meta_labeling.rs`
- [ ] 3.2.2 Implement `MetaLabelingSignal`
- [ ] 3.3.1 Create `src/ml/filtering/base.rs`
- [ ] 3.3.2 Implement `SignalFilter` trait
- [ ] 3.4.1 Create `src/ml/filtering/volatility.rs`
- [ ] 3.4.2 Implement `VolatilityFilter`
- [ ] 3.5.1 Update `src/ml/signals/mod.rs`
- [ ] 3.5.2 Update `src/ml/filtering/mod.rs`

## Summary by Phase

- **Phase 1**: 35 tasks (critical compilation fixes)
- **Phase 2**: 45 tasks (core features)
- **Phase 3**: 60+ tasks (additional features)
- **Total**: ~140 specific implementation tasks

## Completion Tracking

Track overall progress:
- [ ] Phase 1 Complete (33/35)
- [ ] Phase 2 Complete (0/45)
- [ ] Phase 3 Complete (0/60)
- [ ] Project 100% Implemented

## Priority Order

1. **Critical** (Phase 1): Get project compiling
2. **High** (Phase 2): Complete core functionality
3. **Medium** (Phase 3): Add remaining features
