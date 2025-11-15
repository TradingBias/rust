# TradeBias AI - Project Plan

This directory contains a comprehensive project plan to fix all compilation errors and complete all missing features in the TradeBias AI codebase.

## 📋 Current Project Status

- **Build Status**: ❌ Does not compile (86 errors - Polars 0.51.0 migration)
- **Dependency Status**: ✅ hashbrown conflict RESOLVED (see semantic-mapper.md)
- **Implementation**: ~70% structurally complete, needs Polars API migration + fixes
- **Blockers**: Polars 0.51.0 API breaking changes, trait conflicts, type mismatches, missing configuration system

## 📚 Plan Documents

### Core Planning Documents

**⚠️ START HERE FIRST:**

0. **[06-polars-0.51-migration.md](./06-polars-0.51-migration.md)** - **CRITICAL - Must Complete First!** (86 errors)
   - Fix all Polars 0.51.0 API breaking changes
   - Resolve hashbrown dependency conflict (SOLVED)
   - Update Series::new(), datetime accessors, RollingOptions, etc.
   - Goal: Achieve clean `cargo build` before Phase 1
   - **⚡ This is Phase 0 - blocking all other work**

**Then proceed with original phases:**

1. **[00-overview.md](./00-overview.md)** - Project overview
   - Project status summary
   - Plan structure overview
   - Success criteria
   - Estimated scope

2. **[01-phase1-compilation-fixes.md](./01-phase1-compilation-fixes.md)** - Critical (35 tasks)
   - Fix all 50+ compilation errors
   - Resolve trait architecture conflicts
   - Fix Polars API compatibility
   - Goal: Achieve `cargo build` success

3. **[02-phase2-core-features.md](./02-phase2-core-features.md)** - High Priority (45 tasks)
   - Implement configuration system (completely missing)
   - Fix Polars DSL issues in indicators
   - Complete partial implementations
   - Goal: All core features functional

4. **[03-phase3-additional-features.md](./03-phase3-additional-features.md)** - Medium Priority (60+ tasks)
   - Implement 20 Tier 2 indicators
   - Complete data connector framework
   - Implement ML signal/filtering engines
   - Goal: 100% feature completeness

### Reference Documents

5. **[04-quick-reference-checklist.md](./04-quick-reference-checklist.md)**
   - Condensed checklist of all ~140 tasks
   - Progress tracking template
   - Task-by-task completion tracking

6. **[05-implementation-guide.md](./05-implementation-guide.md)**
   - Detailed instructions for AI agents
   - Code style guidelines
   - Common pitfalls and solutions
   - Verification strategies

7. **[semantic-mapper.md](./semantic-mapper.md)**
   - Investigation of hashbrown/polars dependency conflict
   - Root cause analysis and solution discovery
   - Documents the path to Polars 0.51.0 upgrade
   - Historical record of troubleshooting steps

## 🎯 Quick Start

### For Human Project Managers

1. **Start with `06-polars-0.51-migration.md`** - Must complete Phase 0 first!
2. Read `semantic-mapper.md` to understand the hashbrown dependency resolution
3. Read `00-overview.md` to understand the overall plan structure
4. Review `04-quick-reference-checklist.md` for task breakdown
5. Assign phases to AI agents or developers (Phase 0 → Phase 1 → Phase 2 → Phase 3)
6. Track progress using the checklist

### For AI Agents

1. **🚨 CRITICAL: Start with Phase 0** - Read `06-polars-0.51-migration.md` first
2. Complete all 4 stages of Polars 0.51.0 migration (86 errors to fix)
3. Verify `cargo build` succeeds before proceeding
4. Then read `05-implementation-guide.md` thoroughly
5. Continue with Phase 1 tasks in sequential order
6. Follow the code templates and patterns exactly
7. Mark tasks complete in `04-quick-reference-checklist.md`
8. **Do NOT run tests** - only implement code changes

### For Developers

1. **⚠️ START HERE: Complete Phase 0 Polars migration** (`06-polars-0.51-migration.md`)
2. Review current errors: `cargo build 2>&1 | tee build_errors.txt`
3. Follow the 4-stage implementation order in the migration plan
4. Check implementation status in `docs/implementation_summary.md`
5. After Phase 0, pick next phase based on priority
6. Follow the detailed task instructions
7. Verify with `cargo check` after each phase

## 📊 Project Metrics

### Scope
- **Phase 0 (NEW)**: 86 Polars API fixes across ~50 files - **BLOCKING ALL OTHER WORK**
- **Total Tasks**: ~140 specific code changes (original phases)
- **Total Files**: ~100 files to modify (including Phase 0)
- **New Code**: ~3,000-4,000 lines estimated (original phases)
- **Timeline**:
  - Phase 0: 5-7 hours (Polars migration)
  - Phases 1-3: 3-5 days for experienced Rust developer
  - **Total: 4-6 days**

### Task Distribution (Including Phase 0)
```
Phase 0: Polars Migration     86 errors (38%)  ████████████████
Phase 1: Critical Fixes       35 tasks  (16%)  ██████░░░░░░░░░░
Phase 2: Core Features        45 tasks  (20%)  ████████░░░░░░░░
Phase 3: Additional Features  60 tasks  (26%)  ██████████░░░░░░
```

### Priority Breakdown
```
BLOCKING (Phase 0)  ████████████████████████████████  100% MUST-DO FIRST
Critical (Phase 1)  ████████████████████████████████  100% must-do
High (Phase 2)      ████████████████████████░░░░░░░░   80% must-do
Medium (Phase 3)    ████████████░░░░░░░░░░░░░░░░░░░░   40% must-do
```

## 🔍 Key Architectural Decisions

### Decision 0: Polars Version Upgrade (Phase 0) ✅ **COMPLETED**
**Problem**: `raw_table_mut` compilation error due to hashbrown version conflicts
**Solution**: Upgrade to Polars 0.51.0 (resolved hashbrown dependency issue)
**Impact**: 86 API breaking changes require code migration across entire codebase
**Status**: Dependency conflict SOLVED, code migration in progress (see 06-polars-0.51-migration.md)
**Reference**: See semantic-mapper.md for full investigation details

### Decision 1: Trait Architecture (Phase 1, Task 4.1)
**Problem**: Conflicting blanket trait implementations
**Solution**: Use enum-based `StrategyFunction` instead of trait inheritance
**Impact**: Affects registry, semantic mapper, and all function access patterns

### Decision 2: AST Representation (Phase 1, Task 5.1)
**Problem**: Inconsistent use of `StrategyAST` vs `AstNode`
**Recommended**: Have `StrategyAST` wrap `AstNode` with metadata
**Impact**: Affects backtester, evolution engine, and all AST consumers

### Decision 3: Configuration Architecture (Phase 2, Task 1)
**Problem**: System completely missing
**Solution**: Implement trait-based config sections with TOML serialization
**Impact**: Enables runtime configuration without recompilation

## 🚨 Critical Blockers

**⚠️ PHASE 0 MUST BE COMPLETED FIRST:**

0. **Polars 0.51.0 API Migration** (Phase 0 - 86 errors) - **BLOCKS ALL OTHER WORK**
   - See `06-polars-0.51-migration.md` for complete migration plan
   - Must fix all 86 API compatibility errors before proceeding
   - Estimated time: 5-7 hours

**Then resolve these Phase 1 blockers:**

1. **Trait Conflicts** (Task 4.1) - Prevents compilation
2. **AST Type Mismatch** (Task 5.1) - Prevents backtester from working
3. **Remaining Polars API Issues** (Tasks 3.1-3.2) - Prevents indicators from working
4. **Import Errors** (Tasks 1.1-1.7) - Prevents many files from compiling

## ✅ Success Criteria

### Phase 0 Success (Polars Migration) - **MUST COMPLETE FIRST**
- [ ] All 86 Polars 0.51.0 API errors resolved
- [ ] `cargo build` completes without errors
- [ ] All `Series::new()` calls updated to use `.into()`
- [ ] All datetime accessors updated to use `.phys.get()`
- [ ] All `RollingOptions` replaced with `RollingOptionsFixedWindow`
- [ ] All `output_type()` methods added to indicators
- [ ] No Polars API warnings
- [ ] Ready to proceed with Phase 1

### Phase 1 Success
- [ ] `cargo check` returns 0 errors
- [ ] All import paths resolve correctly
- [ ] Trait architecture compiles without conflicts
- [ ] Polars methods are accessible
- [ ] Type system is consistent

### Phase 2 Success
- [ ] Configuration system fully implemented
- [ ] Can load/save config from TOML files
- [ ] All Polars DSL calls work correctly
- [ ] All core features functional
- [ ] `cargo build` still succeeds

### Phase 3 Success
- [ ] All 20 Tier 2 indicators working
- [ ] CSV data connector functional
- [ ] ML signal framework implemented
- [ ] All specs in `docs/ai-implementation/` marked complete
- [ ] Project ready for testing phase

### Final Success
- [ ] All phases complete
- [ ] `cargo build --release` succeeds
- [ ] All items in `implementation_summary.md` marked "Implemented"
- [ ] No TODO comments in critical paths
- [ ] Code ready for `cargo test` (but don't run yet)

## 📖 Related Documentation

- `docs/implementation_summary.md` - Current implementation status
- `docs/errors.md` - Current compilation errors
- `docs/ai-implementation/*.md` - Feature specifications (19 files)
- `Cargo.toml` - Dependencies and project configuration

## 🔄 Execution Workflow

```mermaid
graph TD
    A[Start] --> B[Read semantic-mapper.md & 06-polars-0.51-migration.md]
    B --> C[Phase 0: Polars 0.51 Migration]
    C --> D{cargo build OK?}
    D -->|No - 86 errors| C
    D -->|Yes - 0 errors| E[Read Overview & Implementation Guide]
    E --> F[Phase 1: Compilation Fixes]
    F --> G{cargo check OK?}
    G -->|No| F
    G -->|Yes| H[Phase 2: Core Features]
    H --> I{All core features implemented?}
    I -->|No| H
    I -->|Yes| J[Phase 3: Additional Features]
    J --> K{All features implemented?}
    K -->|No| J
    K -->|Yes| L[Verification]
    L --> M{All criteria met?}
    M -->|No| N[Fix Issues]
    N --> L
    M -->|Yes| O[Complete!]

    style C fill:#ff6b6b,stroke:#c92a2a,stroke-width:3px
    style D fill:#ff6b6b,stroke:#c92a2a,stroke-width:2px
```

## 💡 Tips for Success

1. **🚨 START WITH PHASE 0**: Do NOT skip the Polars migration - it blocks everything else
2. **Follow the 4-Stage Order in Phase 0**: Complete Stage 1 → Stage 2 → Stage 3 → Stage 4
3. **Follow the Phase Order**: Phases are sequenced to minimize rework (0 → 1 → 2 → 3)
4. **One Phase at a Time**: Don't skip ahead or mix phases
5. **Verify Often**:
   - Phase 0: Run `cargo build 2>&1 | grep "error\[E" | wc -l` after each stage
   - Other phases: Run `cargo check` after each major change
6. **Document Assumptions**: Add comments when making design choices
7. **Use Templates**: Follow the code templates in the plan documents
8. **Don't Test Yet**: Focus purely on implementation, testing comes later

## 📞 Getting Help

If you encounter issues not covered in the plan:

1. Check `docs/errors.md` for current error state
2. Review the relevant `docs/ai-implementation/*.md` spec
3. Look for similar patterns in existing code
4. Document the issue and make a reasonable assumption
5. Use `todo!()` macro for complex logic to be filled later

## 🎓 Learning Resources

For understanding the codebase:
- **Polars**: https://pola-rs.github.io/polars/
- **Rust Traits**: https://doc.rust-lang.org/book/ch10-02-traits.html
- **Genetic Programming**: See `docs/ai-implementation/11-evolution-engine.md`
- **Meta-Labeling**: See `docs/ai-implementation/15-ml-meta-labeling.md`

## 📝 License

Same as main project.

---

**Last Updated**: 2025-11-12
**Plan Version**: 1.0
**Status**: Ready for implementation
