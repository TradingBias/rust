# TradeBias AI Implementation Guide - Overview

## Purpose
This directory contains AI-optimized implementation instructions broken down into focused, manageable documents. Each document is designed to be processed independently by AI agents with clear inputs, outputs, and verification steps.

## Document Structure

Follow these documents **in order** for complete implementation:

### Phase 1: Foundation
- **[01-architecture.md](./01-architecture.md)** - Project structure, directory layout, and module dependencies
- **[02-type-system.md](./02-type-system.md)** - Core types, traits, and error handling

### Phase 2: Building Blocks
- **[03-primitives.md](./03-primitives.md)** - All 12 primitive functions (MovingAverage, Highest, Lowest, etc.)

### Phase 3: Indicators
- **[04-indicators-tier1.md](./04-indicators-tier1.md)** - 10 must-have indicators (SMA, EMA, RSI, MACD, etc.)
- **[05-indicators-tier2.md](./05-indicators-tier2.md)** - 20 common indicators (WilliamsR, MFI, SAR, etc.)

### Phase 4: Infrastructure
- **[06-registry-and-cache.md](./06-registry-and-cache.md)** - Function discovery and indicator caching system

### Phase 5: Engines
- **[07-backtesting-engine.md](./07-backtesting-engine.md)** - Expression builder and portfolio simulator
- **[08-metrics-engine.md](./08-metrics-engine.md)** - Performance metrics calculation

### Phase 6: Code Generation
- **[09-code-generation.md](./09-code-generation.md)** - MQL5 EA and indicator library generation

### Phase 7: Quality Assurance
- **[10-testing.md](./10-testing.md)** - Testing strategies, verification, and validation

## Quick Start for AI Agents

### Recommended Approach
1. Read **01-architecture.md** first to understand the overall structure
2. Read **02-type-system.md** to understand the type system before implementing anything
3. Implement in order: Primitives → Tier 1 Indicators → Infrastructure → Engines
4. Run tests from **10-testing.md** after each phase

### Per-Document Structure
Each document contains:
- **Goal**: What this document accomplishes
- **Prerequisites**: What must be completed first
- **Implementation Steps**: Numbered tasks with code examples
- **Verification**: How to validate the implementation
- **Next Steps**: What document to read next

## Key Project Goals

1. **Pure Rust Implementation**: Migrate from Python/Rust hybrid to pure Rust
2. **Custom Indicators**: No MQL5 built-ins, all algorithms implemented from scratch
3. **Vectorized Performance**: Use Polars for 100-1000x faster backtesting
4. **Mathematical Consistency**: Same calculations in Rust backtesting and MQL5 live trading
5. **Indicator Caching**: Cache computed indicators for performance
6. **MQL5 Code Generation**: Generate EA and custom indicator library from Rust implementations

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    egui UI (Frontend)                        │
│              Direct Rust Function Calls                      │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                   Core Rust Library                          │
│  ┌──────────────┬──────────────┬───────────────────────┐   │
│  │   Engines    │  Functions   │   ML Pipeline         │   │
│  │  Generation  │  Indicators  │   Features/Labels     │   │
│  │  Evaluation  │  Primitives  │   Models              │   │
│  │  Metrics     │  Risk        │                       │   │
│  └──────────────┴──────────────┴───────────────────────┘   │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┴──────────────┐
        │                              │
┌───────▼────────┐            ┌────────▼──────────┐
│  Code Generator │            │  Data Connectors  │
│  MQL5 EA + MQH  │            │  CSV / Supabase   │
└────────────────┘            └───────────────────┘
```

## Implementation Strategy

### Two-Mode Approach for Indicators

**Vectorized Mode (Preferred)**:
- Use Polars operations for fast backtesting
- Examples: SMA, EMA, RSI, MACD, ATR, Bollinger Bands
- 100-1000x faster than bar-by-bar

**Stateful Mode (When Needed)**:
- Bar-by-bar calculation with state buffer
- Use when vectorization isn't mathematically practical
- Example: ADX, SAR (complex conditional logic)

**MQL5 Generation**:
- ALL indicators generate stateful MQL5 code (for live trading)
- Vectorized indicators convert their logic to stateful form for codegen

### 12 Primitive Building Blocks

All 70+ indicators are built from these 12 primitives:
1. **MovingAverage** (SMA, EMA, LWMA, SMMA)
2. **Highest** (Max over period)
3. **Lowest** (Min over period)
4. **Sum** (Summation)
5. **StdDev** (Standard deviation)
6. **Momentum** (Price change over period)
7. **Shift** (Time-shift series)
8. **Absolute** (Absolute value)
9. **Divide** (Safe division)
10. **Multiply**
11. **Add**
12. **Subtract**

## Common Pitfalls for AI Agents

1. **Don't skip verification steps** - Each phase has verification steps; run them!
2. **Follow the order** - Later phases depend on earlier ones
3. **Read prerequisites** - Each document lists what must be completed first
4. **Test incrementally** - Don't wait until the end to test
5. **Match code examples exactly** - Especially trait signatures and type definitions
6. **Don't mix vectorized and stateful** - Each indicator uses ONE mode, not both
7. **Cache indicator results** - Use the caching system to avoid recomputation

## Progress Tracking

Use this checklist as you complete each document:

- [ ] 01-architecture.md - Project structure set up
- [ ] 02-type-system.md - Core types and traits implemented
- [ ] 03-primitives.md - All 12 primitives working
- [ ] 04-indicators-tier1.md - 10 core indicators implemented
- [ ] 05-indicators-tier2.md - 20 common indicators implemented
- [ ] 06-registry-and-cache.md - Registry and caching working
- [ ] 07-backtesting-engine.md - Backtesting engine functional
- [ ] 08-metrics-engine.md - Metrics calculation working
- [ ] 09-code-generation.md - MQL5 generation working
- [ ] 10-testing.md - All tests passing

## Questions or Issues?

If implementation details are unclear:
1. Check the **Prerequisites** section - you may need to complete an earlier document
2. Check the **Verification** section - it shows expected outputs
3. Review the **Code Examples** - they show complete, working implementations
4. Refer back to the original `docs/implementation.md` for additional context

## Next Steps

Begin with **[01-architecture.md](./01-architecture.md)** to set up the project structure.
