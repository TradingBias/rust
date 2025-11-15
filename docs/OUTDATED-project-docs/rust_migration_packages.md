# TradeBias Python → Rust Package Migration

## Core Data Processing

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **Polars** | Vectorized data operations | `polars` | Native Rust implementation available via `polars` crate |
| **Pandas** | Data manipulation (fallback) | `polars` or `arrow-rs` | Polars is faster; Arrow for compatibility |
| **NumPy** | Numerical arrays | `ndarray` | N-dimensional array operations |
| **SciPy** | Scientific computing | `ndarray`, `nalgebra` | Linear algebra & statistics |

## Genetic Programming & Evolution

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **DEAP** | Genetic programming framework | `evolving` or custom | No direct equivalent; consider custom impl + `rand` for RNG |
| **Random (stdlib)** | RNG for GP | `rand` | Standard crate for random number generation |
| `random.shuffle()` | Population operations | `rand::seq::SliceRandom` | Trait-based shuffling in Rust |

## Machine Learning & Statistical Analysis

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **scikit-learn** | ML models (ensemble base) | `smartcore` | Linfa is "sklearn for Rust"; smartcore lighter |
| **XGBoost** | Gradient boosting | `xgboost` (via FFI) or `smartcore` | FFI binding available; or use smartcore::tree |
| **RandomForest** | Ensemble classifier | `smartcore::ensemble` | Built-in RandomForest in smartcore |
| **SVM** | Support vector machine | `smartcore::svm` | Available in smartcore |
| **Logistic Regression** | Binary classification | `smartcore::linear` | Standard in smartcore |

## Feature Engineering & Data Labels

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **Triple-barrier labeling** | Signal labeling | Custom impl | Simple enough to rewrite in Rust |
| **Stationarity tests** | Time-series validation | Custom + `statrs` | Use `statrs` for statistical functions |
| **Feature selection (RFECV)** | Feature importance | `smartcore` or custom | Smartcore has feature selection utilities |
| **Correlation analysis** | Feature redundancy | `statrs` or `ndarray` | Matrix operations with ndarray + statrs |

## Configuration Management

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **Pydantic** | Request validation & schemas | `serde`, `serde_json` | Serde handles serialization + derive validation |
| **ConfigParser/TOML** | Config file parsing | `serde_json`, `toml`, `config` | Use `config` crate for unified approach |
| **Python dicts** | Dynamic config | `serde_json::json!` macro | JSON values work well for dynamic data |

## IPC & API Communication

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **FastAPI** | HTTP API server | `axum`, `actix-web`, `rocket` | Axum is modern; Rocket user-friendly; Actix fastest |
| **JSON-RPC** | stdin/stdout protocol | `jsonrpc-lite` | Lightweight JSON-RPC library |
| **stdin/stdout piping** | Process IPC | `std::process::Stdio` | Built-in Rust, no library needed |
| **threading** | Background tasks | `tokio`, `std::thread` | Tokio for async; std::thread for simplicity |

## Database & Storage

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **Supabase SDK** | Cloud storage & DB | `supabase-rs` or `reqwest` + manual | `supabase-rs` is community; otherwise use `reqwest` for HTTP |
| **PostgreSQL** | Metadata storage | `sqlx`, `tokio-postgres` | SQLx for compile-time safety; tokio-postgres for async |
| **REST API calls** | External services | `reqwest` | Async HTTP client, very popular in Rust |

## Serialization & Data Formats

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **JSON (stdlib)** | Message serialization | `serde_json` | De facto standard in Rust |
| **pickle** | Strategy persistence | `bincode` or `serde_json` | Bincode for binary; JSON for human-readable |
| **CSV (pandas)** | Data file loading | `csv` or `polars::io::csv` | Polars for full pipeline; `csv` for lightweight |

## Logging & Debugging

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **logging module** | Application logging | `tracing`, `log` + `env_logger` | `log` is simple; `tracing` is modern & structured |
| **traceback** | Error handling | `anyhow`, `thiserror` | Anyhow for error chains; thiserror for custom types |

## Type System & Validation

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **typing** | Type hints | Built-in | Rust's type system is stronger; no runtime needed |
| **Enum (Python)** | Enumerations | `enum` (Rust) | Native language feature |
| **dataclass** | Data structures | `struct` + `serde::Deserialize` | Serde derive macro replaces dataclass |

## Testing & Benchmarking

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **pytest** | Unit testing | `cargo test` | Built-in Rust testing framework |
| **hypothesis** | Property testing | `proptest`, `quickcheck` | Proptest for sophisticated property tests |
| **timeit** | Benchmarking | `criterion` | Criterion.rs for statistical benchmarking |

## GUI & Frontend Components

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| *(Already Rust)* | Frontend integration | **Keep `egui`** | Continue using egui for UI; remove Python bridge |
| **IPC bridge** | Frontend-backend comm | **Remove entirely** | Rust backend calls Rust functions directly (no JSON needed) |
| **threading** | Background work | `tokio` | Use tokio tasks instead of Python threading |

## Time & Date Handling

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **datetime** | Date/time operations | `chrono` | Industry standard in Rust |
| **time** | Time measurements | `std::time::Instant` | Built-in; or `tokio::time` for async |

## Utilities & Helpers

| Python | Purpose | Rust Replacement | Notes |
|--------|---------|------------------|-------|
| **itertools** | Iterator utilities | Standard library or `itertools` crate | Rust iterators are excellent; crate available if needed |
| **functools** | Caching/decorators | `once_cell`, `lazy_static` | Memoization via lazy initialization |
| **hashlib** | Hashing | `sha2`, `blake3` | Cryptographic hashing crates |
| **multiprocessing** | Parallel computation | `rayon` | Data-parallel library; or `tokio` for async |

---

## Migration Strategy by Layer

### Layer 1: Data & Compute (Highest Priority)

**Python → Rust**
- Polars operations: ✅ Native Rust support via `polars` crate
- Indicators: Rewrite pure calculation logic in Rust (no dependencies needed, just math)
- Metrics calculations: Rewrite as Rust functions over Polars DataFrames
- Backtester: Rewrite core simulation loop in Rust

**Recommended Stack**:
```toml
[dependencies]
polars = { version = "0.19", features = ["lazy", "csv"] }
ndarray = "0.15"
rand = "0.8"
chrono = "0.4"
```

### Layer 2: ML Pipeline (Medium Priority)

**Python → Rust**
- Feature engineering: Custom Rust functions + Polars
- Labeling: Custom triple-barrier implementation in Rust
- Model training: Use `smartcore` or `xgboost` (FFI)

**Recommended Stack**:
```toml
smartcore = "0.3"  # For RF, SVM, LR; consider xgboost for GBM
statrs = "0.16"    # Statistical functions
```

### Layer 3: Configuration & Serialization (Quick Wins)

**Python → Rust**
- Config system: Replace Pydantic + custom config with `serde` + `serde_json`
- Request validation: Use `serde::Deserialize` with custom validators
- Message passing: `serde_json` for JSON serialization

**Recommended Stack**:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
config = "0.13"
```

### Layer 4: IPC Removal (Architecture Simplification)

**Before (Python/Rust Split)**:
```
egui (Rust) ──[JSON-RPC]──> Python Backend
```

**After (Full Rust)**:
```
egui (Rust) ──[Direct Function Calls]──> Core Engines (Rust)
                                        └─ No serialization overhead!
```

**Impact**: Remove `jsonrpc-lite`, stdin/stdout pipes entirely. Direct in-process calls.

**Recommended Stack**:
```toml
# No JSON-RPC needed! Just library organization.
# Create lib.rs modules: engines, functions, ml, etc.
```

### Layer 5: API & Future Integration (As-Needed)

**If keeping API server for external integrations**:

```toml
# Choose one:
axum = "0.7"           # Modern, fast, flexible
actix-web = "4"        # Fastest; heavier
rocket = "0.5"         # Developer-friendly
# For JSON-RPC over HTTP:
jsonrpc-lite = "0.6"
```

---

## Full Rust Stack Recommendation

```toml
[dependencies]
# Core data processing
polars = { version = "0.19", features = ["lazy", "csv", "json"] }
arrow = "51"  # For Arrow integration

# Numerical computing
ndarray = "0.15"
nalgebra = "0.33"

# Genetic Programming (custom + utilities)
rand = "0.8"
rand_distr = "0.4"

# Machine Learning
smartcore = "0.3"
statrs = "0.16"

# Serialization & Config
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
config = "0.13"

# Time handling
chrono = "0.4"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Frontend (keep existing)
egui = "0.24"
eframe = "0.24"

# Testing & Benchmarking
criterion = { version = "0.5", optional = true }
proptest = { version = "1.0", optional = true }

# Async runtime (if needed for API server)
tokio = { version = "1", features = ["full"], optional = true }
```

---

## Effort Estimate by Component

| Component | Lines of Code | Effort | Priority |
|-----------|---------------|--------|----------|
| Indicator library | ~3,000 | 3-4 days | High |
| Backtester | ~1,500 | 2-3 days | High |
| Metrics engine | ~1,000 | 1-2 days | High |
| Genetic algorithm | ~2,000 | 3-5 days | High |
| Configuration system | ~800 | 1 day | Medium |
| ML pipeline | ~2,500 | 4-6 days | Medium |
| API layer (if kept) | ~600 | 1-2 days | Low |
| Testing suite | ~2,000 | 3-4 days | Medium |
| **Total** | **~13,400** | **~18-27 days** | — |

---

## Key Advantages of Full-Rust Migration

1. **No IPC overhead**: Direct function calls, no JSON serialization
2. **Type safety**: Compile-time checking across entire codebase
3. **Performance**: 2-5x faster than Python for compute-heavy operations
4. **Single language**: Simplified build, deployment, maintenance
5. **Memory efficiency**: Better GC, smaller binary with Nuitka
6. **Deployment**: Single binary with embedded UI (egui)

## Key Challenges

1. **Learning curve**: Rust borrow checker; async/await patterns
2. **Ecosystem maturity**: Some Python ML libraries don't have Rust equivalents (but `smartcore` covers 80% of needs)
3. **Development speed**: Type checking + compilation takes longer than Python iteration
4. **Debugging**: Rust error messages can be cryptic; stack traces less readable

---

## Suggested Migration Path

```
Phase 1: Core Engine (Weeks 1-2)
  └─ Indicators → Rust (using polars)
  └─ Backtester → Rust
  └─ Metrics → Rust

Phase 2: Evolution (Week 3)
  └─ Genetic algorithm → Rust
  └─ AST structures → Rust
  └─ Semantic validation → Rust

Phase 3: Integration (Week 4)
  └─ Remove Python backend entirely
  └─ Remove IPC layer
  └─ Direct Rust function calls from egui UI

Phase 4: ML Enhancement (Weeks 5-6)
  └─ ML pipeline → Rust (smartcore)
  └─ Feature engineering → Rust
  └─ Meta-labeling → Rust

Phase 5: Polish (Week 7)
  └─ Testing & benchmarking
  └─ Binary compilation (Nuitka → cargo build --release)
  └─ Deployment scripts
```

---

## Side-by-Side Example: Backtester

### Python Version
```python
# src/core/engines/evaluation/backtester.py
def run(self, ast: dict, data: pl.DataFrame) -> StrategyResult:
    condition_expr = self.expression_builder.build(ast['condition'])
    trades, equity_curve = self.portfolio_simulator.simulate(
        condition_expr, risk_exprs, data
    )
    metrics = self.metrics_engine.calculate_all(...)
    return StrategyResult(ast, metrics, trades, equity_curve)
```

### Rust Version
```rust
// src/engines/evaluation/backtester.rs
impl Backtester {
    pub fn run(&self, ast: &StrategyAst, data: &DataFrame) -> Result<StrategyResult> {
        let condition_expr = self.expression_builder.build(&ast.condition)?;
        let (trades, equity_curve) = self.portfolio_simulator.simulate(
            &condition_expr, &self.risk_exprs, data
        )?;
        let metrics = self.metrics_engine.calculate_all(&trades, &equity_curve)?;
        Ok(StrategyResult {
            ast: ast.clone(),
            metrics,
            trades,
            equity_curve,
        })
    }
}
```

The structure is identical; Rust just adds compile-time safety!