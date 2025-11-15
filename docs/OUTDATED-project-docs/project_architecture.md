# TradeBias Project Architecture

> **Purpose**: This document explains the TradeBias project structure, what each file does, its role in the system, and how everything connects. Written to help AI agents quickly understand the codebase.

**Project**: TradeBias v0.1.0
**Type**: Hybrid Python/Rust algorithmic trading strategy generation and backtesting platform with ML enhancement
**Core Technology**:
- Backend: Python with Polars (vectorized data operations), Genetic Programming, AST manipulation
- Frontend: Rust with egui (native GUI framework)
- Data Storage: Supabase (cloud storage for market data)
- ML Stack: Scikit-learn, XGBoost for meta-labeling and signal filtering

---

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [Directory Structure](#directory-structure)
3. [Source Code (`src/`) Deep Dive](#source-code-src-deep-dive)
4. [Library Config (`library/config/`) Deep Dive](#library-config-libraryconfig-deep-dive)
5. [Data Flow & System Architecture](#data-flow--system-architecture)
6. [Key Concepts](#key-concepts)

---

## High-Level Overview

TradeBias is a **genetic programming-based trading strategy generator** that:

1. **Generates** trading strategies using grammar-based genetic programming
2. **Validates** strategies using semantic type checking and AST (Abstract Syntax Tree) validation
3. **Executes** strategies using vectorized Polars expressions for performance
4. **Evaluates** strategies through backtesting with comprehensive metrics
5. **Exports** strategies to MQL5 and Python code for deployment

### Core Architecture Pattern

```
User Request → API Layer → Evolution Engine → Strategy Generation → Backtesting → Metrics → Results
                                ↓
                         Function Registry (Indicators + Primitives)
                                ↓
                         AST Builder → Expression Builder → Polars Execution
```

---

## Directory Structure

```
tradebias/
├── src/                          # Python backend source code
│   ├── main_api.py               # Main entry point: JSON IPC interface
│   ├── main.py                   # Alternative entry point
│   ├── api/                      # API layer (NEW)
│   │   ├── market_data.py        # FastAPI server for market data
│   │   ├── validation.py         # Request validation and security
│   │   └── schemas.py            # Pydantic request/response schemas
│   └── core/                     # Core system components
│       ├── __init__.py           # Version info
│       ├── events.py             # Event system for IPC
│       ├── config/               # Configuration management
│       ├── data/connectors/      # Data source connectors (CSV, MT5, API)
│       ├── compute/              # Metrics computation backends
│       ├── engines/              # Core execution engines
│       │   ├── generation/       # Strategy generation (GP, semantic mapping)
│       │   ├── evaluation/       # Backtesting and portfolio simulation
│       │   ├── validation/       # Robustness testing
│       │   ├── metrics_engine.py # Performance metrics calculation
│       │   ├── signal_engine.py  # Signal extraction from strategies
│       │   └── calibration_engine.py # Auto-calibration of indicator ranges
│       ├── functions/            # Trading functions (indicators, primitives, risk)
│       ├── utilities/            # Helper utilities
│       ├── interpreter/          # Type checking, expression building, validation
│       └── ml/                   # Machine Learning module for strategy filtering
│           ├── features/         # Feature engineering
│           ├── meta_labeling/    # Triple-barrier labeling and meta-models
│           ├── validation/       # Purged cross-validation
│           ├── models/           # ML model zoo and calibration
│           ├── portfolio/        # Bet sizing and attribution
│           └── evaluation/       # Performance analysis
│
├── ui/                           # Rust frontend (NEW)
│   ├── src/
│   │   ├── main.rs               # Application entry point
│   │   ├── lib.rs                # Library root
│   │   ├── python_bridge.rs     # IPC bridge to Python backend
│   │   ├── types/                # Rust type definitions
│   │   │   ├── mod.rs
│   │   │   └── market_data.rs    # Market data types
│   │   └── app/                  # Application components
│   │       ├── mod.rs            # Main app state and routing
│   │       ├── state.rs          # Application state definitions
│   │       └── components/       # UI components
│   │           ├── config_panel.rs          # Generate tab config
│   │           ├── central_panel.rs         # Main results display
│   │           ├── strategy_detail_panel.rs # Strategy details
│   │           ├── ml_config_panel.rs       # ML/Refine tab config
│   │           ├── ml_results_panel.rs      # ML results display
│   │           ├── market_data_browser.rs   # Market data browser
│   │           └── ohlc_chart.rs            # OHLC chart viewer
│   └── Cargo.toml                # Rust dependencies
│
└── library/                      # Static configuration files
    └── config/
        ├── indicator_metadata.json    # Indicator compatibility/semantic info
        └── pattern_manifest.json      # Strategy pattern templates
```

---

## Source Code (`src/`) Deep Dive

### 1. Entry Points

#### `src/main_api.py`
**Role**: Main entry point for the application. Provides a comprehensive JSON-based IPC (Inter-Process Communication) interface.

**Key Functions**:
- `main()`: Infinite loop reading JSON commands from stdin, routing to command handlers
- `get_project_version()`: Returns version string
- `load_data_source()`: Loads and validates CSV data with security checks
- `run_evolution()`: Dispatches the evolution/genetic programming process
- `get_function_lists()`: Returns available indicators and primitives
- `get_metrics_list()`: Returns available fitness metrics
- `get_base_models_list()`: Returns available ML base models
- `get_feature_types_list()`: Returns available feature types

**ML Pipeline Functions**:
- `extract_signals()`: Extract all signals from a strategy AST
- `engineer_features()`: Create features for ML training
- `label_signals()`: Label signals using triple-barrier method
- `train_meta_model()`: Train ensemble ML model for signal filtering
- `predict_signal_quality()`: Predict signal quality with trained model
- `backtest_with_ml_filter()`: Run backtest with ML signal filtering
- `compare_strategies()`: Compare performance of multiple strategies
- `analyze_ml_improvement()`: Comprehensive ML improvement analysis

**Market Data Functions**:
- `get_market_data_files()`: List market data files from Supabase
- `get_market_data_symbols()`: Get unique symbols from market data
- `get_market_data_file_url()`: Get public URL for market data file
- `get_ohlc_data()`: Download and parse OHLC data

**Connections**:
- Imports: Core modules, ML modules, Supabase client
- Outputs: JSON responses to stdout for frontend consumption
- Input format: `{"id": 1, "command": "...", "payload": {...}}`
- Security: Request validation, file path sanitization, size limits

#### `src/api/market_data.py`
**Role**: FastAPI server for market data access (optional HTTP API alternative to IPC).

**Endpoints**:
- `GET /api/market-data/files`: List market data files with filtering
- `GET /api/market-data/download/{file_path}`: Download a market data file
- `GET /api/market-data/file-url/{file_path}`: Get public URL for file
- `GET /api/market-data/symbols`: List all unique symbols

**Features**:
- CORS enabled for Rust frontend
- Streaming file downloads
- Supabase Storage integration

---

### 2. Configuration System (`src/core/config/`)

#### `src/core/config/manager.py`
**Role**: Singleton configuration manager that centralizes all application settings.

**Key Features**:
- Loads defaults from unified manifest
- Provides dot-notation access (e.g., `config.get('performance.backend')`)
- Type validation and parsing

**Connections**:
- Depends on: `core.config.ui.builder.get_unified_manifest()`
- Used by: Most system components for settings

#### `src/core/config/ui/builder.py`
**Role**: Aggregates all UI manifest modules into a single unified configuration structure.

**Key Function**:
- `get_unified_manifest()`: Merges manifests from settings, data_source, evolution, trade_management, backtesting, function_set, and fitness modules

**Manifest Modules** (all in `src/core/config/ui/`):
- `settings.py`: General application settings (performance, workers, etc.)
- `data_source.py`: Data file path and source configuration
- `evolution.py`: Genetic algorithm parameters (population, generations, mutation rate, etc.)
- `trade_management.py`: Risk management settings (stop-loss, take-profit, position sizing)
- `backtesting.py`: Backtest configuration (validation method, data split)
- `function_set.py`: Available indicators for strategy building
- `fitness.py`: Fitness objectives for evolution

**Structure**: Each module returns a nested dict of settings with metadata like type, default, editor component, validation rules.

#### `src/core/config/metrics_manifest.py`
**Role**: Defines all available performance metrics and their dependencies.

**Key Function**:
- `get_metrics_manifest()`: Returns dictionary mapping metric keys to metadata

**Metric Categories**:
- Profitability: net profit, ROI, CAGR
- Trade Statistics: win rate, expectancy, average trade duration
- Positional Statistics: long/short performance breakdown
- Risk & Drawdown: max drawdown, VaR, volatility
- Risk-Adjusted Returns: Sharpe, Sortino, Calmar ratios
- Cost Analysis: commissions, fees

**Dependency System**: Each metric lists its dependencies (e.g., `sharpe_ratio` depends on `equity_curve`, `risk_free_rate`, `periods_per_year`). This enables topological calculation ordering.

---

### 3. Function System (`src/core/functions/`)

The function system is the heart of strategy building. It provides a registry of all available trading components.

#### `src/core/functions/functions_registry.py`
**Role**: Auto-discovers and registers all primitives and indicators by introspecting `BasePrimitive` subclasses.

**Key Class**: `StrategyFunctionRegistry`

**Methods**:
- `__init__(library_path, allowed_indicators)`: Discovers all functions
- `_discover_all()`: Recursively finds all `BasePrimitive` subclasses
- `_register_class()`: Instantiates and categorizes functions
- `get_all_functions()`: Returns flat dict of all functions by alias
- `get_functions_by_output_type(type, arity)`: Type-aware function lookup for semantic generation

**Structure**:
- `primitives_by_category`: Dict of categories → {alias: instance}
- `indicators_by_category`: Dict of categories → {alias: instance}

**Nuitka-Compatible**: Uses class introspection instead of file-system scanning for compiled builds.

---

#### Function Categories

##### A. Primitives (`src/core/functions/primitives/`)

###### `base.py`
**Role**: Base class for all strategy components.

**Key Constants**:
- Execution modes: `EXECUTION_MODE_VECTORIZED`, `EXECUTION_MODE_STATEFUL`, `EXECUTION_MODE_DATA_SOURCE`
- Data types: `D_TYPE_NUMERIC_SERIES`, `D_TYPE_BOOL_SERIES`, `D_TYPE_INTEGER`, `D_TYPE_FLOAT`, `D_TYPE_GENERIC`

**BasePrimitive Attributes**:
- `alias`: Unique identifier (e.g., "SMA", "GreaterThan")
- `ui_name`: Display name for UI
- `category`: Grouping (e.g., "trend", "comparison")
- `execution_mode`: How it executes
- `arity`: Number of arguments
- `input_types`: List of expected input data types
- `output_type`: Type of value returned
- `input_config`: UI configuration for arguments

**Methods**:
- `__call__(*args)`: Returns Polars expression
- `generate_mql5(inputs)`: Generates MQL5 code
- `generate_python(inputs)`: Generates Python code

###### Primitive Modules:
- `action.py`: Trading actions (OpenLong, OpenShort, ClosePosition)
- `comparison.py`: Comparison operators (GreaterThan, LessThan, Equals) with safe type coercion
- `crossover.py`: Crossover detection (CrossAbove, CrossBelow)
- `data.py`: Market data accessors (Open, High, Low, Close, Volume)
- `logical.py`: Boolean logic (And, Or, Not)
- `math.py`: Mathematical operations (Add, Subtract, Multiply, Divide)
- `time.py`: Time-based filters (IsHour, IsDayOfWeek, etc.)

##### B. Indicators (`src/core/functions/indicators/`)

Each indicator is a separate file implementing a specific technical indicator.

**Example**: `sma.py` (Simple Moving Average)
```python
class SMA(BasePrimitive):
    alias = "SMA"
    ui_name = "Simple Moving Average"
    category = "trend"
    arity = 2
    input_types = [D_TYPE_NUMERIC_SERIES, D_TYPE_INTEGER]
    output_type = D_TYPE_NUMERIC_SERIES

    def __call__(self, series_expr, period_expr):
        period = extract_literal(period_expr)
        return series_expr.rolling_mean(window_size=period)
```

**Categories**:
- **Trend**: SMA, EMA, LWMA, SMMA, DEMA, TEMA, AMA, VIDYA, SAR
- **Momentum**: RSI, Stochastic, MACD, CCI, Momentum, WPR, TriX, DeMarker, RVI
- **Volatility**: ATR, StdDev, ADX, Bears/Bulls Power
- **Volume**: OBV, MFI, AD, Force, Volumes, Chaikin, BWMFI
- **Bands**: Bollinger Bands, Envelopes
- **Ichimoku**: Tenkan, Kijun, Senkou A/B, Chikou
- **Bill Williams**: Alligator, AC, AO, Gator, Fractals

##### C. Risk Functions (`src/core/functions/risk/`)

**Base**: `base.py` - `BaseRiskPrimitive` class

**Modules**:
- `stop_loss.py`: StopLossAtrMultiplier, StopLossFixed
- `take_profit.py`: TakeProfitRiskRewardRatio, TakeProfitFixed
- `position_sizing.py`: PositionSizeRiskPercent, PositionSizeFixed

---

### 4. Engines (`src/core/engines/`)

#### A. Data Contracts (`data_contracts.py`)
**Role**: Pydantic models defining data structures passed between components.

**Key Models**:
- `Trade`: Single completed trade (entry/exit bar, price, direction, size, profit, exit_reason, fees)
- `StrategyResult`: Complete backtest output (ast, metrics, trades, equity_curve, wfo_folds)
- `ResultPackage`: In-sample + out-of-sample results bundle

#### B. Metrics Engine (`metrics_engine.py`)
**Role**: Calculates all performance metrics from backtest results.

**Key Class**: `MetricsEngine`

**Constructor Args**:
- `backend`: BaseMetricsBackend implementation (Polars or Pandas)
- `config`: Trade management config
- `asset_metadata`: Timeframe info for annualization

**Key Method**: `calculate_all(result: StrategyResult) → Dict[str, float]`

**Process**:
1. Extract trades DataFrame and equity curve
2. Build dependency graph from metrics manifest
3. Topologically sort and calculate metrics in order
4. Each metric calls its backend method with required dependencies
5. Return flattened dict of scalar metrics (filters out Series/DataFrames)

**Dependency Resolution**: Iteratively calculates metrics once all dependencies are available. Supports chained dependencies (e.g., Calmar depends on CAGR which depends on equity_curve).

---

#### C. Generation Engine (`src/core/engines/generation/`)

The generation engine creates trading strategies using genetic programming.

##### `expression_builder.py`
**Role**: Converts AST (Abstract Syntax Tree) to executable Polars expressions.

**Key Class**: `ExpressionBuilder`

**Method**: `build(ast: dict) → pl.Expr`

**Process**:
1. Recursively walks AST nodes
2. For `"const"` nodes: converts to `pl.lit(value)`
3. For `"call"` nodes: looks up function in registry, recursively builds arguments, calls function
4. Returns composed Polars expression

**Type Safety**: All scalars are converted to Polars literals for type consistency.

##### `semantic_mapper.py`
**Role**: Generates semantically valid strategy ASTs from genomes using type-driven recursive generation.

**Key Classes**:
- `GeneConsumer`: Deterministically consumes genes from genome list
- `SemanticMapper`: Type-aware AST builder

**Key Method**: `create_strategy_ast(genome: list[int]) → dict`

**Process**:
1. Consume genes to make deterministic choices
2. Start with desired output type (D_TYPE_BOOL_SERIES for conditions)
3. Recursively select compatible functions and build arguments
4. For integer/float arguments: map genes to value ranges
5. Respect max_depth to prevent infinite trees
6. Return complete AST with condition + action nodes

**Type System**: Ensures generated strategies are semantically valid by matching input/output types.

##### `hall_of_fame.py`
**Role**: Maintains best-performing strategies across generations.

**Features**:
- Configurable size limit
- Automatic deduplication (using canonical AST strings)
- Filtering by minimum performance thresholds
- Sorting by fitness objectives

##### `lightweight_validator.py`
**Role**: Fast pre-execution validation of strategy ASTs.

**Checks**:
- AST structure validity
- Function existence in registry
- Type compatibility
- Arity matching
- Recursion depth limits

##### `argument_diversity_validator.py`
**Role**: Validates that indicator parameters are diverse enough.

**Purpose**: Prevents generating strategies with redundant indicators (e.g., SMA(14) and SMA(15)).

##### `smart_parameter_generator.py`
**Role**: Generates intelligent parameter values for indicators based on metadata.

**Features**:
- Uses indicator_metadata.json for valid ranges
- Considers indicator scale types
- Generates appropriate thresholds for comparisons

##### `grammar_generator.py`
**Role**: Builds grammar rules for strategy generation.

**Uses**: `library/config/pattern_manifest.json` to define valid structural patterns.

##### `evolution_engine.py`
**Role**: Main genetic programming loop (not fully shown in excerpts, but inferred).

**Expected Functions**:
- Population initialization
- Fitness evaluation (via backtesting)
- Selection, crossover, mutation operations
- Generation loop with Hall of Fame updates
- Progress reporting via IPC callbacks

##### Optimisation Submodules (`optimisation/`)

**Methods** (`methods/`):
- `base_method.py`: Abstract base for validation methods
- `simple_method.py`: Single train/test split
- `wfo_method.py`: Walk-Forward Optimization (rolling windows)

**Splitters** (`splitters/`):
- `base_splitter.py`: Abstract base for data splitting
- `simple_splitter.py`: Simple percentage split
- `wfo_splitter.py`: Walk-forward rolling window splitter

---

#### D. Evaluation Engine (`src/core/engines/evaluation/`)

##### `backtester.py`
**Role**: High-level orchestrator for complete backtest execution.

**Key Class**: `Backtester`

**Constructor Args**:
- `expression_builder`: ExpressionBuilder instance
- `portfolio_simulator`: PortfolioSimulator instance
- `metrics_engine`: MetricsEngine instance
- `config`: Trade management config

**Key Method**: `run(ast: dict, data: pl.DataFrame) → StrategyResult`

**Process**:
1. Build condition expression from AST
2. Build risk expressions (stop-loss, take-profit, position sizing) from config
3. Pass expressions to portfolio simulator
4. Receive trades and equity curve
5. Pass to metrics engine for performance calculation
6. Bundle into StrategyResult

**Risk Expression Building**: Dynamically constructs Polars expressions for risk management based on config (e.g., ATR-based stops, risk-reward ratio take-profits).

##### `portfolio_simulator.py`
**Role**: Vectorized trade simulation using Polars expressions.

**Key Class**: `PortfolioSimulator`

**Key Method**: `simulate(condition_expr, risk_exprs, data) → (trades, equity_curve)`

**Process**:
1. Evaluate condition expression over full dataset (vectorized)
2. Identify entry signals
3. For each entry:
   - Calculate stop-loss and take-profit levels
   - Find exit bar (first to hit SL/TP or data end)
   - Calculate P&L, fees, slippage
   - Update equity
4. Build equity curve series
5. Return trades list and equity curve

**Vectorization**: Uses Polars lazy evaluation for performance on large datasets.

---

#### E. Validation Engine (`src/core/engines/validation/`)

**Role**: Provides a centralized, pluggable system for running post-discovery robustness tests on a single, user-selected strategy. This module is triggered by the user to generate a "Robustness Report" for a strategy from the Hall of Fame.

##### `orchestrator.py`
**Role**: Main entry point to run the full "Robustness Report".

**Key Class**: `ValidationOrchestrator`

**Key Method**: `run_robustness_report(strategy_ast: dict, data: pl.DataFrame, progress_callback: Callable)`

**Process**:
1.  Initializes all robustness tests defined in `robustness_methods/`.
2.  Receives the `strategy_ast` of the chosen strategy.
3.  Iterates through each test, calling its `run()` method.
4.  Aggregates the results into a single, comprehensive JSON-serializable report.

##### `robustness_methods/base_test.py`
**Role**: Abstract base class for all robustness tests.

**Key Class**: `AbstractRobustnessTest(ABC)`

**Abstract Method**: `run(self, strategy_ast: dict, data: pl.DataFrame, backtester: Backtester, **kwargs) -> dict`

##### `robustness_methods/monte_carlo.py`
**Role**: Implements the **Monte Carlo (Trade Permutation)** test.

**Key Class**: `MonteCarloTradeTest`

**Process**:
1.  Runs the original backtest to get the list of trade profits.
2.  Shuffles the trade profits multiple times to create new equity curves.
3.  Calculates statistics on the distribution of simulated outcomes.

##### `robustness_methods/friction_test.py`
**Role**: Implements the **Delayed Entry/Exit Test** to simulate slippage and latency.

**Key Class**: `FrictionTest`

**Process**:
1.  Shifts the entry and exit signals by one bar.
2.  Runs the portfolio simulator with the delayed signals.
3.  Compares the metrics of the original and delayed backtests.

##### `robustness_methods/parameter_stability.py`
**Role**: Implements the **System Parameter Permutation (SPP)** test.

**Key Class**: `ParameterStabilityTest`

**Process**:
1.  Parses the strategy AST to find all numeric parameters.
2.  Generates a range of new values for each parameter.
3.  Runs a backtest for each modified strategy.
4.  Returns a list of results to analyze parameter sensitivity.

---

### 5. Connectors (`src/core/connector/`)

#### `csv_connector.py`
**Role**: Loads and validates CSV data files.

**Key Class**: `CsvConnector`

**Key Method**: `load_and_validate(file_path) → (pl.DataFrame, metadata)`

**Process**:
1. Read CSV with pandas for flexible date parsing
2. Normalize column names (lowercase, handle variations)
3. Detect and parse datetime column
4. Validate each row against `OhlcvRecord` Pydantic model
5. Convert to Polars DataFrame
6. Return data + metadata (symbol, start/end dates, row count)

**Validation Model**: `OhlcvRecord` - ensures datetime, open, high, low, close, volume columns with correct types.

#### `mt5_connector.py`
**Role**: Connects to MetaTrader 5 platform for live/historical data.

**Expected Features** (not fully shown):
- MT5 API initialization
- Symbol data retrieval
- Timeframe conversion
- Real-time tick handling

#### `api_connector.py`
**Role**: Generic API connector for web-based data sources.

**Expected Features**:
- REST API requests
- Authentication handling
- Rate limiting
- Data normalization

#### `factory.py`
**Role**: Factory pattern for connector instantiation.

**Expected Function**: `get_connector(source_type) → Connector`

---

### 6. Compute Backends (`src/core/compute/`)

#### `base_metrics.py`
**Role**: Abstract base class for metrics computation backends.

**Key Class**: `BaseMetricsBackend`

**Abstract Methods**: One method per metric (e.g., `sharpe_ratio()`, `max_drawdown()`, etc.)

**Purpose**: Allows swapping between Polars and Pandas implementations for metrics calculation.

#### `metrics_backend.py`
**Role**: Concrete implementation of metrics backend (likely Polars-based).

**Key Class**: `PolarsMetricsBackend` (inferred name)

**Implements**: All metric calculations using Polars operations for performance.

**Example Methods**:
- `sharpe_ratio(equity_curve, risk_free_rate, periods_per_year)`: Annualized Sharpe ratio
- `max_drawdown_pct(equity_curve)`: Maximum percentage drawdown
- `win_rate_pct(trades_df, total_trades)`: Percentage of winning trades

---

### 7. Utilities (`src/core/utilities/`)

#### `ast_converter.py`
**Role**: Converts AST dictionaries to human-readable string representations.

**Key Functions**:
- `get_canonical_ast_string(node)`: Deterministic JSON representation for duplicate detection
- `AstConverter.ast_to_string(ast)`: Converts AST to readable format (e.g., "(RSI(Close, 14) > 70)")

**Robustness**: Never fails; uses placeholders for malformed nodes.

#### `code_generator.py`
**Role**: Generates deployable code from ASTs.

**Key Class**: `CodeGenerator`

**Methods**:
- `generate_mql5(ast)`: Complete MQL5 Expert Advisor
- `generate_python(ast)`: Executable Python strategy function

**Template Structure**:
- MQL5: Full EA with OnInit, OnTick, position management
- Python: Strategy function taking data, portfolio, current bar

#### `strategy_signature.py`
**Role**: Generates unique signatures for strategies (likely for hashing/comparison).

#### `decorators.py`
**Role**: Useful decorators for the codebase (e.g., timing, caching).

#### `logging.py`
**Role**: Centralized logging configuration.

**Key Function**: `get_logger(name) → Logger`

**Features**:
- Module-level loggers
- Configurable log levels
- Consistent formatting

#### `settings_manager.py`
**Role**: Legacy settings management (likely superseded by config.manager).

#### `time_data_mapper.py`
**Role**: Maps time periods to data points based on timeframe.

**Use Case**: Converting "5 days" to number of bars for different timeframes (1H, 4H, D1, etc.).

#### `mp_utils.py`
**Role**: Multiprocessing utilities for parallel evolution.

**Expected Features**:
- Pool management
- Process-safe logging
- Shared memory handling

#### `ray_utils.py`
**Role**: Ray framework utilities for distributed computing (if used).

---

### 8. Events (`src/core/events.py`)
**Role**: Event system for IPC communication with frontend.

**Expected Features**:
- Event emission
- Progress updates
- Error reporting
- Structured message formatting

---

### 9. API Layer (`src/api/`)

**Role**: Provides validation, security, and schema definitions for API requests.

#### `src/api/validation.py`
**Role**: Request validation and security enforcement.

**Key Functions**:
- `validate_file_path()`: Sanitizes and validates file paths
- `validate_file_size()`: Enforces file size limits
- `validate_api_version()`: Version compatibility checking
- `format_error_response()`: Standardized error formatting
- `get_cache()`: In-memory cache for data and models

**Security Features**:
- Path traversal prevention
- Extension whitelisting
- File size limits (default 100MB)
- Sandbox mode support

#### `src/api/schemas.py`
**Role**: Pydantic models for request/response validation.

**Key Models**:
- `LoadDataSourceRequest`: Data loading request
- `DataMetadata`: Data file metadata
- `GetStrategyDetailsRequest`: Strategy detail request
- `StrategyDetails`: Strategy detail response
- `RunBacktestRequest`: Backtest request

---

### 10. Machine Learning Module (`src/core/ml/`)

**Role**: Implements a secondary machine learning layer (meta-model) to improve the performance of strategies generated by the genetic programming engine. It filters trade signals, sizes positions, and provides advanced performance analysis, based on the concepts from "Advances in Financial Machine Learning" by de Prado.

**Core Architecture Pattern**:
```
GP Strategy Signal → Triple-Barrier Labeling → Feature Engineering → Meta-Model Training → ML Filter → Bet Sizing → Execution
```

#### A. Meta-Labeling (`ml/meta_labeling/`)
**Role**: Generates labels for training the meta-model.
- `labeler.py`: Implements the triple-barrier method to label signals based on profit targets, stop losses, and time expiration.
- `signal_generator.py`: Extracts entry signals from the primary GP-generated strategies.
- `meta_model.py`: Trains an ensemble of classifiers (e.g., XGBoost, RandomForest) on the generated features and labels to predict the probability of a label being profitable.

#### B. Features (`ml/features/`)
**Role**: Creates features for the meta-model from market data.
- `feature_engineer.py`: Generates a wide range of features, including price action, volatility, volume, momentum, and time-based features. Designed to prevent lookahead bias.
- `feature_selector.py`: Provides methods for feature selection, such as mutual information, recursive feature elimination (RFECV), and correlation-based redundancy removal.
- `stationary.py`: Implements fractional differentiation to make time-series features stationary.

#### C. Validation (`ml/validation/`)
**Role**: Provides robust cross-validation techniques tailored for financial time-series data.
- `purged_cv.py`: Implements Combinatorial Purged Cross-Validation (CPCV) to prevent data leakage between training and testing sets by purging overlapping samples and applying an embargo period.
- `sample_weights.py`: Creates sample weights based on uniqueness (to address overlapping labels), returns, and time decay.

#### D. Models (`ml/models/`)
**Role**: Manages the machine learning models used in the meta-model ensemble.
- `model_zoo.py`: A factory for creating various classifiers (XGBoost, RandomForest, SVM, etc.) with standardized interfaces and default hyperparameters.
- `calibration.py`: Provides tools for probability calibration (Platt Scaling, Isotonic Regression) to ensure model outputs are reliable probabilities.
- `base_classifier.py`: Defines the abstract base classes and wrappers for all models, ensuring a consistent API.

#### E. Portfolio (`ml/portfolio/`)
**Role**: Applies the ML model's output to portfolio management.
- `bet_sizing.py`: Sizes positions based on the meta-model's prediction confidence (e.g., using a variant of the Kelly Criterion).
- `attribution.py`: Calculates strategy and feature contribution to portfolio performance using Shapley values.
- `risk_limits.py`: Implements dynamic risk management, including drawdown control, leverage limits, and correlation-based concentration limits.

#### F. Evaluation (`ml/evaluation/`)
**Role**: Evaluates the performance of the ML-filtered strategies.
- `metrics.py`: Calculates ML-specific metrics like precision, recall, ROC-AUC, and calibration error, in addition to standard trading metrics.
- `performance_analyzer.py`: Provides tools to compare the performance of the raw GP strategy against the ML-filtered version, analyzing improvements and providing detailed reports.

---

## Rust Frontend (`ui/`) Architecture

**Role**: Native desktop GUI built with Rust and egui, communicating with Python backend via JSON IPC over stdin/stdout.

### Architecture Pattern

```
User Interface (egui) → App State → Python Bridge → IPC (stdin/stdout) → Python Backend
                          ↓
                    Component Rendering
```

### Key Components

#### `ui/src/main.rs`
**Role**: Application entry point and window configuration.

**Key Functions**:
- `main()`: Initializes eframe native window
- Window configuration: title, size, persistence

#### `ui/src/lib.rs`
**Role**: Library root, exports main modules.

**Modules**:
- `app`: Application state and UI
- `python_bridge`: IPC communication
- `types`: Type definitions

#### `ui/src/python_bridge.rs`
**Role**: Manages bidirectional IPC with Python backend.

**Key Features**:
- Spawns Python process with stdin/stdout pipes
- Sends JSON commands to Python backend
- Receives responses and progress updates
- Thread-safe message passing via mpsc channels
- Auto-generates unique command IDs

**Key Methods**:
- `new(tx: Sender<AppMessage>)`: Initialize bridge with message sender
- `send_command(command: &str, payload: Value)`: Send command to Python
- Background thread continuously reads Python responses

#### `ui/src/app/mod.rs`
**Role**: Main application state, tab routing, and message handling.

**Key Structures**:
- `MyApp`: Main application state
- `Tab` enum: Generate, Refine, MarketData

**State Categories**:
- **Evolution State**: `hall_of_fame`, `is_running_evolution`, `evolution_status`
- **Generate Tab**: `generate_settings`, `generate_dynamic_data`
- **Refine Tab**: `ml_settings`, `refine_dynamic_data`, `ml_model_state`
- **Market Data**: `market_data_state`
- **Strategy Details**: `selected_strategy_id`, `strategy_detail_data`

**Message Handling**:
- Processes `AppMessage` from Python bridge
- Updates UI state based on responses
- Handles errors and progress updates

#### `ui/src/app/state.rs`
**Role**: Type definitions for application state.

**Key Types**:
- `UnifiedManifest`: Dynamic configuration from Python
- `HallOfFameEntry`: Strategy from evolution
- `StrategyData`: Detailed strategy information
- `SignalData`: Extracted trading signals
- `DataMetadata`: Loaded data file metadata
- `MLModelState`: ML model training state
- `LabeledFeatures`: Features with labels for ML
- `TrainingResult`: ML training results
- `MLComparisonData`: ML vs baseline comparison
- `GenerateTabSettings`: User settings for Generate tab
- `GenerateTabDynamicData`: Dynamic data (indicators, metrics, primitives)
- `RefineTabDynamicData`: Dynamic data (base models, feature types)
- `MLSettings`: ML configuration settings
- `MarketDataBrowserState`: Market data browser state
- `AppMessage` enum: All message types from Python

#### `ui/src/app/components/config_panel.rs`
**Role**: Configuration panel for Generate tab.

**Features**:
- Data source selection
- Evolution parameters (population, generations, etc.)
- Function set selection (indicators, primitives)
- Fitness objectives selection
- Trade management settings (stop loss, take profit, position sizing)
- Backtest configuration
- Run evolution button

**UI Elements**:
- Collapsing sections for organization
- Dynamic lists loaded from Python
- Modal dialogs for multi-select
- Real-time validation

#### `ui/src/app/components/central_panel.rs`
**Role**: Main results display for Generate tab.

**Features**:
- Hall of Fame table
- Strategy ranking and metrics
- Selection handling
- Progress indicators

#### `ui/src/app/components/strategy_detail_panel.rs`
**Role**: Detailed strategy view panel.

**Features**:
- AST visualization
- In-sample and out-of-sample metrics
- Trade list
- Equity curve chart
- Code generation (MQL5, Python)

#### `ui/src/app/components/ml_config_panel.rs`
**Role**: Configuration panel for Refine tab (ML training).

**Features**:
- Signal extraction settings
- Feature engineering configuration
- Labeling configuration (triple-barrier method)
- Model configuration (base models, ensemble method)
- Training configuration (CV strategy, splits)
- Step-by-step ML pipeline execution

**Workflow**:
1. Extract signals from strategy
2. Engineer features
3. Label signals
4. Train meta-model
5. Backtest with ML filter

#### `ui/src/app/components/ml_results_panel.rs`
**Role**: Results display for Refine tab.

**Features**:
- Training metrics display
- Feature importance visualization
- Model performance comparison
- ML vs baseline comparison charts

#### `ui/src/app/components/market_data_browser.rs`
**Role**: Browse and download market data from Supabase.

**Features**:
- File list with filtering (symbol, asset type)
- Search functionality
- Symbol list loading
- File preview and download
- OHLC chart visualization
- Real-time loading indicators

**UI Elements**:
- Filter controls (symbol, asset type)
- Search box
- Refresh button
- Files table with sortable columns
- Chart viewer (integrated)

#### `ui/src/app/components/ohlc_chart.rs`
**Role**: OHLC candlestick chart viewer.

**Features**:
- Candlestick rendering
- Time axis formatting
- Price axis formatting
- Zoom and pan (via egui_plot)
- Volume display (optional)
- Interactive tooltips

**Chart Components**:
- Candlestick bodies (green/red)
- High/low wicks
- Time labels
- Price labels
- Grid lines

#### `ui/src/types/market_data.rs`
**Role**: Type definitions for market data.

**Key Types**:
- `MarketDataFile`: Market data file record from Supabase
- `MarketDataFilter`: Filter parameters for queries
- `OHLCData`: Single OHLC bar (timestamp, OHLC, volume)
- `OHLCDataset`: Complete dataset with symbol and timeframe

### UI Architecture Patterns

1. **Immediate Mode GUI**: Uses egui's immediate mode pattern - UI rebuilt every frame
2. **State Management**: Centralized state in `MyApp`, passed to components
3. **Component Pattern**: Render functions take `&mut egui::Ui` and `&mut MyApp`
4. **Async Communication**: Background threads for Python IPC, message passing for updates
5. **Persistence**: Uses eframe persistence for user settings
6. **Modal Dialogs**: egui::Window for selection modals
7. **Layout**: egui panels (TopBottom, Side, Central) for layout structure

### IPC Message Flow

```
User Action → Component → App State Update → Python Bridge
                                                   ↓
                                           Command Sent (JSON)
                                                   ↓
                                           Python Backend Processes
                                                   ↓
                                           Response Sent (JSON)
                                                   ↓
Background Thread → mpsc::Receiver → AppMessage → State Update → UI Re-render
```

---

## Library Config (`library/config/`) Deep Dive

### `indicator_metadata.json`
**Role**: Central metadata for indicator compatibility, scales, and semantic information.

**Structure**:
```json
{
  "_metadata": {...},
  "_scale_types": {
    "price": {...},
    "oscillator_0_100": {...},
    "oscillator_centered": {...},
    "volatility_decimal": {...},
    "volume": {...},
    "ratio": {...},
    "index": {...}
  },
  "indicators": {
    "SMA": {
      "full_name": "Simple Moving Average",
      "scale": "price",
      "value_range": null,
      "category": "trend"
    },
    "RSI": {
      "full_name": "Relative Strength Index",
      "scale": "oscillator_0_100",
      "value_range": [0, 100],
      "category": "momentum"
    },
    ...
  }
}
```

**Purpose**:
- **Scale Compatibility**: Defines which indicators can be meaningfully compared
- **Value Ranges**: Expected output ranges for validation
- **Categories**: Semantic grouping (trend, momentum, volatility, volume)
- **Threshold Generation**: Helps smart_parameter_generator create appropriate thresholds

**Scale Types**:
- `price`: Price-following (MAs, Bollinger Bands) - compatible with price data
- `oscillator_0_100`: Bounded 0-100 (RSI, Stochastic) - use percentage thresholds
- `oscillator_centered`: Zero-centered (MACD, Momentum) - use zero-crossing logic
- `volatility_decimal`: Small decimals (ATR, StdDev) - special threshold scaling
- `volume`: Volume-based - large integer thresholds
- `ratio`: Ratio-based (Williams %R)
- `index`: Index-based (ADX, CCI)

### `pattern_manifest.json`
**Role**: Defines valid structural patterns for strategy generation.

**Structure**:
```json
[
  {
    "name": "oscillator_vs_trend",
    "structure": {
      "operator": {"type": "primitive", "category": ["comparison", "crossover"]},
      "lhs": {"type": "indicator", "category": "oscillator"},
      "rhs": {"type": "indicator", "category": "trend"}
    }
  },
  {
    "name": "oscillator_vs_constant_threshold",
    "structure": {
      "operator": {"type": "primitive", "category": "comparison"},
      "lhs": {"type": "indicator", "category": "oscillator"},
      "rhs": {"type": "number", "range": [20, 80]}
    }
  },
  ...
]
```

**Purpose**:
- **Grammar Rules**: Defines valid strategy patterns (e.g., "RSI > 70" is valid, "RSI > SMA" might not be)
- **Type Constraints**: Ensures semantically meaningful comparisons
- **Parameter Ranges**: Guides value generation for constants

**Pattern Types**:
- Indicator vs Indicator: Different category comparisons
- Indicator vs Price: Direct price comparisons
- Indicator vs Threshold: Constant value comparisons
- Normalized Comparisons: Using normalization functions
- Specialized Patterns: Price distance from MA, Z-score extremes

---

## Data Flow & System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Rust Frontend (UI)                     │
│  ┌──────────────┬──────────────┬─────────────────────────┐ │
│  │ Generate Tab │  Refine Tab  │  Market Data Browser    │ │
│  │  (GP + GP)   │  (ML Filter) │  (Supabase Integration) │ │
│  └──────────────┴──────────────┴─────────────────────────┘ │
│                           ↕ IPC (JSON)                      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Python Backend (Core)                    │
│  ┌──────────────────┬───────────────────┬────────────────┐ │
│  │ Evolution Engine │  ML Pipeline      │ Market Data    │ │
│  │ (GP, Backtest)   │  (Meta-labeling)  │ (Supabase API) │ │
│  └──────────────────┴───────────────────┴────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      External Services                       │
│  ┌──────────────────┬───────────────────┐                  │
│  │ Supabase Storage │  Supabase DB      │                  │
│  │ (Market Data)    │  (Metadata)       │                  │
│  └──────────────────┴───────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Complete Flow: Evolution Run (Generate Tab)

```
1. USER REQUEST (Rust UI)
   └─> User clicks "Run Evolution" in config_panel.rs
       └─> App sends {"command": "run_evolution", "payload": {...}} via PythonBridge

2. PYTHON BACKEND RECEIVES REQUEST (main_api.py)
   └─> main() loop reads JSON from stdin
       └─> Routes to run_evolution() handler

3. INITIALIZATION
   ├─> CsvConnector loads data from file (with validation)
   ├─> StrategyFunctionRegistry discovers all functions
   ├─> CalibrationEngine runs auto-calibration on data
   └─> Config parsed and validated

4. EVOLUTION LOOP (for each generation)
   ├─> Population Initialization/Mutation
   │   ├─> SemanticMapper.create_strategy_ast(genome)
   │   │   └─> Type-driven recursive AST building
   │   └─> LightweightValidator validates AST structure
   │
   ├─> Evaluation (for each strategy in population)
   │   ├─> ExpressionBuilder.build(ast)
   │   │   └─> Converts AST to Polars expressions
   │   │
   │   ├─> Backtester.run(ast, data)
   │   │   ├─> Builds condition expression
   │   │   ├─> Builds risk expressions (SL, TP, sizing)
   │   │   ├─> PortfolioSimulator.simulate()
   │   │   │   └─> Vectorized trade simulation
   │   │   ├─> MetricsEngine.calculate_all()
   │   │   │   └─> Topological dependency resolution
   │   │   └─> Returns StrategyResult
   │   │
   │   └─> Fitness calculation from metrics
   │
   ├─> Selection & Genetic Operators
   │   ├─> Tournament/roulette selection
   │   ├─> Crossover (AST subtree exchange)
   │   └─> Mutation (node replacement, subtree generation)
   │
   ├─> Hall of Fame Update
   │   ├─> Deduplication via canonical AST strings
   │   └─> Filtering by thresholds
   │
   └─> Progress Updates
       └─> Send {"stream": "progress", "type": "...", "payload": {...}} to stdout

5. RUST UI RECEIVES UPDATES
   ├─> PythonBridge background thread reads stdout
   ├─> Parses JSON and creates AppMessage
   ├─> Sends AppMessage via mpsc channel
   └─> MyApp.update() receives message and updates state
       └─> UI re-renders with new data

6. OUTPUT
   ├─> Best strategies returned in Hall of Fame
   ├─> CodeGenerator.generate_mql5() for deployment
   └─> User can select strategies for detailed view or ML refinement
```

### Complete Flow: ML Refinement (Refine Tab)

```
1. SIGNAL EXTRACTION
   User clicks "Extract Signals" → extract_signals() command
   ├─> SignalGenerator extracts all entry signals from strategy AST
   ├─> Adds indicator values at each signal
   └─> Returns signals DataFrame

2. FEATURE ENGINEERING
   User clicks "Engineer Features" → engineer_features() command
   ├─> FeatureEngineer creates features from signals and market data
   │   ├─> Price-based features (returns, volatility)
   │   ├─> Volume-based features
   │   ├─> Momentum indicators
   │   └─> Pattern recognition features
   └─> Returns features DataFrame

3. SIGNAL LABELING
   User clicks "Label Signals" → label_signals() command
   ├─> TripleBarrierLabeler applies triple-barrier method
   │   ├─> Sets profit target (upper barrier)
   │   ├─> Sets stop loss (lower barrier)
   │   ├─> Sets time limit (vertical barrier)
   │   └─> Labels: 1 (profit), -1 (loss), 0 (timeout)
   └─> Returns labeled_data DataFrame with distribution stats

4. MODEL TRAINING
   User clicks "Train Model" → train_meta_model() command
   ├─> MetaModel initializes ensemble
   │   ├─> Base models (XGBoost, RandomForest, etc.)
   │   ├─> Ensemble method (logistic, voting, stacking)
   │   └─> Calibration method (isotonic, platt)
   ├─> Cross-validation (Purged CV)
   ├─> Trains ensemble on features and labels
   ├─> Calculates training metrics (accuracy, precision, recall, AUC)
   └─> Returns model_id and training_metrics

5. BACKTESTING WITH ML FILTER
   User clicks "Backtest with ML" → backtest_with_ml_filter() command
   ├─> Predicts probability for each signal
   ├─> Filters signals by probability threshold
   ├─> Runs backtest on filtered signals
   └─> Compares to baseline (unfiltered) strategy

6. PERFORMANCE ANALYSIS
   User views results → compare_strategies() / analyze_ml_improvement()
   ├─> PerformanceAnalyzer compares baseline vs ML-filtered
   ├─> Calculates improvement metrics
   ├─> Generates comparison charts
   └─> Displays in ml_results_panel.rs
```

### Complete Flow: Market Data Access (Market Data Tab)

```
1. BROWSE FILES
   User opens Market Data tab → get_market_data_files() command
   ├─> Queries Supabase table "market_data_files"
   ├─> Applies filters (symbol, asset_type)
   ├─> Returns list of MarketDataFile records
   └─> Displayed in market_data_browser.rs table

2. LOAD SYMBOLS
   User clicks "Load Symbols" → get_market_data_symbols() command
   ├─> Queries unique symbols from Supabase
   ├─> Returns sorted list of symbols
   └─> Populates filter dropdown

3. VIEW CHART
   User clicks file row → get_ohlc_data() command
   ├─> Downloads JSON file from Supabase Storage
   ├─> Parses and validates OHLC data
   ├─> Converts timestamps to Unix time
   ├─> Returns OHLCDataset
   └─> Renders in ohlc_chart.rs with egui_plot

4. DOWNLOAD FILE
   User clicks "Download" → get_market_data_file_url() command
   ├─> Gets public URL from Supabase Storage
   ├─> Opens URL in browser (or downloads directly)
   └─> User can use file for local backtesting
```

### Abstraction Layers

```
Layer 1: USER INTERFACE (Rust Frontend)
         ├─> egui immediate mode GUI
         ├─> MyApp (application state)
         ├─> Components (config_panel, central_panel, etc.)
         └─> PythonBridge (IPC communication)
                    ↓ JSON over stdin/stdout

Layer 2: API & SECURITY (Python Backend Entry)
         ├─> main_api.py (command routing)
         ├─> api/validation.py (security, validation)
         ├─> api/schemas.py (Pydantic models)
         └─> api/market_data.py (FastAPI server)

Layer 3: CONFIGURATION & DISCOVERY
         ├─> config.manager (settings)
         ├─> config.ui.* (UI manifests)
         └─> StrategyFunctionRegistry (function discovery)

Layer 4: STRATEGY REPRESENTATION
         ├─> AST dictionaries (abstract representation)
         ├─> functions_registry (available components)
         ├─> ast_converter (human-readable strings)
         └─> code_generator (AST → MQL5/Python)

Layer 5: STRATEGY GENERATION (Genetic Programming)
         ├─> SemanticMapper (AST creation from genomes)
         ├─> Grammar rules from pattern_manifest.json
         ├─> Type system (D_TYPE_* constants)
         ├─> Validation (lightweight_validator, argument_diversity)
         └─> EvolutionEngine (genetic algorithm)

Layer 6: STRATEGY EXECUTION (Backtesting)
         ├─> ExpressionBuilder (AST → Polars expressions)
         ├─> Backtester (orchestration)
         ├─> PortfolioSimulator (trade simulation)
         ├─> Data connectors (CSV, MT5, API)
         └─> CalibrationEngine (auto-calibration)

Layer 7: PERFORMANCE EVALUATION
         ├─> MetricsEngine (calculation orchestration)
         ├─> metrics_manifest (metric definitions)
         ├─> BaseMetricsBackend (computation)
         └─> data_contracts (result structures)

Layer 8: ML PIPELINE (Meta-Labeling & Signal Filtering)
         ├─> SignalEngine (signal extraction)
         ├─> FeatureEngineer (feature creation)
         ├─> TripleBarrierLabeler (labeling)
         ├─> MetaModel (ensemble training)
         ├─> PerformanceAnalyzer (comparison)
         └─> Portfolio management (bet sizing, risk)

Layer 9: OPTIMIZATION & VALIDATION
         ├─> HallOfFame (elite preservation)
         ├─> Optimization methods (Simple, WFO)
         ├─> Data splitters (train/test splitting)
         └─> ValidationEngine (robustness checks)

Layer 10: EXTERNAL SERVICES
         ├─> Supabase Storage (market data files)
         ├─> Supabase DB (metadata)
         └─> Future: Live data feeds, broker APIs
```

---

## Key Concepts

### 1. Abstract Syntax Tree (AST)
**Definition**: Dictionary representation of strategy logic as a tree structure.

**Node Types**:
- `"rule"`: Root node with condition and action
- `"call"`: Function call with arguments
- `"const"`: Constant value (integer, float, string)

**Example**:
```python
{
  "node_type": "rule",
  "condition": {
    "node_type": "call",
    "function": "GreaterThan",
    "args": [
      {
        "node_type": "call",
        "function": "RSI",
        "args": [
          {"node_type": "call", "function": "Close", "args": []},
          {"node_type": "const", "value": 14}
        ]
      },
      {"node_type": "const", "value": 70}
    ]
  },
  "action": {
    "node_type": "call",
    "function": "OpenLong",
    "args": []
  }
}
```
**Represents**: `if RSI(Close, 14) > 70 then OpenLong`

### 2. Type System
**Purpose**: Ensures semantic validity of generated strategies.

**Core Types**:
- `D_TYPE_NUMERIC_SERIES`: Polars numeric column (prices, indicator values)
- `D_TYPE_BOOL_SERIES`: Polars boolean column (conditions, signals)
- `D_TYPE_INTEGER`: Scalar integer (periods, thresholds)
- `D_TYPE_FLOAT`: Scalar float (multipliers, percentages)
- `D_TYPE_GENERIC`: Catch-all for flexible types

**Type Checking**: SemanticMapper ensures output type of one function matches input type of next.

### 3. Vectorized Execution (Polars)
**Why Polars?**: Blazing fast columnar operations, lazy evaluation, type safety.

**Pattern**: Instead of row-by-row loops:
```python
# SLOW (row-by-row)
for i in range(len(data)):
    if data['close'][i] > data['open'][i]:
        signals[i] = 1

# FAST (vectorized)
signals = pl.col('close') > pl.col('open')
```

**Benefits**:
- 10-100x faster for large datasets
- Memory efficient (lazy evaluation)
- Type-safe operations

### 4. Dependency Resolution (Metrics)
**Problem**: Some metrics depend on other metrics (e.g., Calmar ratio needs CAGR and max_drawdown).

**Solution**: Topological sorting using dependency graph.

**Process**:
1. Start with metrics that only depend on raw data (equity_curve, trades_df)
2. Calculate these first
3. Add results to calculation_inputs
4. Proceed to metrics that now have all dependencies available
5. Repeat until all metrics calculated

### 5. Walk-Forward Optimization (WFO)
**Purpose**: More robust out-of-sample validation by simulating live trading conditions.

**Process**:
1. Split data into multiple overlapping windows
2. For each window:
   - Train on in-sample period
   - Test on out-of-sample period
3. Roll window forward
4. Aggregate results across all folds

**Benefits**: Reduces overfitting, tests strategy adaptability to changing markets.

### 6. Genetic Programming
**Core Idea**: Evolution-inspired optimization of tree-structured programs (strategies).

**Operators**:
- **Selection**: Choose best performers (tournament, roulette)
- **Crossover**: Swap subtrees between two strategies
- **Mutation**: Replace random node/subtree with new content
- **Elitism**: Preserve best strategies (Hall of Fame)

**Fitness**: Multi-objective based on selected metrics (Sharpe, profit, drawdown, etc.).

### 7. Semantic Awareness
**Problem**: Naive GP generates nonsensical strategies (e.g., "RSI > Close").

**Solutions**:
1. **Type System**: Enforce input/output type compatibility
2. **Scale Metadata**: Only compare compatible indicators (e.g., price-scale vs price-scale)
3. **Grammar Rules**: Pattern manifest defines valid structures
4. **Argument Diversity**: Prevent redundant parameters (e.g., SMA(14) vs SMA(15))

### 8. IPC (Inter-Process Communication)
**Why?**: Python backend with computational power, Rust frontend for native UI performance.

**Protocol**: Line-delimited JSON over stdin/stdout.

**Architecture**:
- Rust UI spawns Python process with stdin/stdout pipes
- Background thread in PythonBridge continuously reads Python stdout
- Commands sent to Python stdin, responses/progress from Python stdout
- Thread-safe message passing via mpsc channels

**Message Types**:

**Commands (Rust → Python)**:
```json
{
  "id": 1,
  "command": "run_evolution",
  "payload": {
    "population_size": 50,
    "generations": 10,
    ...
  }
}
```

**Responses (Python → Rust)**:
```json
{
  "id": 1,
  "command": "run_evolution",
  "payload": {...},
  "error": null
}
```

**Progress Updates (Python → Rust)**:
```json
{
  "stream": "progress",
  "type": "generation_complete",
  "payload": {
    "generation": 5,
    "best_fitness": 1.23
  }
}
```

**Errors (Python → Rust)**:
```json
{
  "stream": "error",
  "payload": "Traceback (most recent call last):\n..."
}
```

**Available Commands**:
- Core: `get_project_version`, `load_data_source`, `get_strategy_details`, `run_backtest`
- Evolution: `run_evolution`
- ML Pipeline: `extract_signals`, `engineer_features`, `label_signals`, `train_meta_model`, `predict_signal_quality`, `backtest_with_ml_filter`, `compare_strategies`, `analyze_ml_improvement`
- Market Data: `get_market_data_files`, `get_market_data_symbols`, `get_market_data_file_url`, `get_ohlc_data`
- Dynamic Lists: `get_function_lists`, `get_metrics_list`, `get_base_models_list`, `get_feature_types_list`
- Cache: `get_cached_data_info`
- Model Management: `get_feature_importance`, `save_ml_model`, `load_ml_model`

---

## Quick Reference: File → Purpose

### Python Backend

| File | One-Line Purpose |
|------|------------------|
| `main_api.py` | Main JSON IPC entry point for all commands |
| `api/market_data.py` | FastAPI server for market data HTTP endpoints |
| `api/validation.py` | Request validation and security enforcement |
| `api/schemas.py` | Pydantic models for request/response validation |
| `config/manager.py` | Singleton settings manager with dot-notation access |
| `config/ui/builder.py` | Aggregates all UI manifests into unified config |
| `config/metrics_manifest.py` | Defines all metrics and their dependencies |
| `functions/functions_registry.py` | Auto-discovers and catalogs all strategy components |
| `functions/primitives/base.py` | Base class for all functions (indicators, operators, etc.) |
| `functions/indicators/*.py` | Individual indicator implementations (SMA, RSI, etc.) |
| `functions/risk/*.py` | Risk management functions (SL, TP, position sizing) |
| `engines/metrics_engine.py` | Calculates all performance metrics with dependency resolution |
| `engines/generation/expression_builder.py` | Converts AST to executable Polars expressions |
| `engines/generation/semantic_mapper.py` | Generates type-valid ASTs from genomes |
| `engines/generation/hall_of_fame.py` | Maintains best strategies across generations |
| `engines/generation/evolution_engine.py` | Main genetic programming loop |
| `engines/evaluation/backtester.py` | Orchestrates complete backtest execution |
| `engines/evaluation/portfolio_simulator.py` | Vectorized trade simulation |
| `engines/validation/orchestrator.py` | Runs a full robustness report on a single strategy |
| `engines/validation/robustness_methods/base_test.py` | Abstract base class for all robustness tests |
| `engines/validation/robustness_methods/monte_carlo.py` | Implements the Monte Carlo (Trade Permutation) test |
| `engines/validation/robustness_methods/friction_test.py` | Implements the Delayed Entry/Exit Test to simulate slippage |
| `engines/validation/robustness_methods/parameter_stability.py` | Implements the System Parameter Permutation (SPP) test |
| `engines/data_contracts.py` | Pydantic models for results (Trade, StrategyResult, etc.) |
| `connector/csv_connector.py` | Loads and validates CSV data files |
| `compute/base_metrics.py` | Abstract base for metrics computation backends |
| `compute/metrics_backend.py` | Concrete metrics implementation (Polars) |
| `utilities/ast_converter.py` | Converts AST to human-readable strings |
| `utilities/code_generator.py` | Generates MQL5/Python code from AST |
| `utilities/logging.py` | Centralized logging configuration |
| `ml/meta_labeling/labeler.py` | Implements triple-barrier labeling for meta-model training |
| `ml/features/feature_engineer.py` | Generates features for the meta-model |
| `ml/validation/purged_cv.py` | Implements Combinatorial Purged Cross-Validation (CPCV) |
| `ml/models/model_zoo.py` | Factory for creating ML models for the meta-model ensemble |
| `ml/portfolio/bet_sizing.py` | Sizes positions based on ML model confidence |
| `ml/evaluation/performance_analyzer.py` | Compares raw vs. ML-filtered strategy performance |
| `library/config/indicator_metadata.json` | Indicator scale/category/compatibility metadata |
| `library/config/pattern_manifest.json` | Valid strategy structure patterns (grammar rules) |
| `engines/signal_engine.py` | Extracts trading signals from strategy ASTs |
| `engines/calibration_engine.py` | Auto-calibrates indicator parameter ranges from data |

### Rust Frontend

| File | One-Line Purpose |
|------|------------------|
| `ui/src/main.rs` | Application entry point, window initialization |
| `ui/src/lib.rs` | Library root, module exports |
| `ui/src/python_bridge.rs` | IPC bridge to Python backend via stdin/stdout |
| `ui/src/app/mod.rs` | Main application state, tab routing, message handling |
| `ui/src/app/state.rs` | Type definitions for application state |
| `ui/src/app/components/config_panel.rs` | Configuration panel for Generate tab |
| `ui/src/app/components/central_panel.rs` | Main results display for Generate tab |
| `ui/src/app/components/strategy_detail_panel.rs` | Detailed strategy view with metrics and trades |
| `ui/src/app/components/ml_config_panel.rs` | Configuration panel for Refine tab (ML) |
| `ui/src/app/components/ml_results_panel.rs` | ML results display and comparison |
| `ui/src/app/components/market_data_browser.rs` | Browse and download market data from Supabase |
| `ui/src/app/components/ohlc_chart.rs` | OHLC candlestick chart viewer |
| `ui/src/types/market_data.rs` | Market data type definitions |
| `ui/Cargo.toml` | Rust dependencies and project configuration |

---

## Common Tasks & Where to Look

### Task: Add a New Indicator
1. Create file in `src/core/functions/indicators/` (e.g., `my_indicator.py`)
2. Subclass `BasePrimitive`, set attributes (alias, arity, input_types, output_type)
3. Implement `__call__(*args) → pl.Expr`
4. Implement `generate_mql5(inputs) → str`
5. Add metadata to `library/config/indicator_metadata.json`
6. Registry auto-discovers it on next run

### Task: Add a New Metric
1. Add method to `compute/metrics_backend.py` (e.g., `def my_metric(self, **deps)`)
2. Add entry to `config/metrics_manifest.py` with dependencies
3. MetricsEngine automatically includes it in calculation

### Task: Modify Strategy Generation Logic
1. Adjust type rules in `functions/primitives/base.py` (D_TYPE_* constants)
2. Modify `engines/generation/semantic_mapper.py` (type-driven generation)
3. Update `library/config/pattern_manifest.json` (grammar rules)
4. Tweak `engines/generation/argument_diversity_validator.py` (parameter validation)

### Task: Change Risk Management
1. Modify `config/ui/trade_management.py` (UI config)
2. Update `engines/evaluation/backtester.py` (_build_risk_expressions method)
3. Add/modify functions in `functions/risk/` if new risk types needed

### Task: Add New Data Source
1. Create connector in `connector/` subclassing abstract connector interface
2. Implement `load_and_validate()` method returning (pl.DataFrame, metadata)
3. Update `connector/factory.py` to recognize new source type
4. Update `config/ui/data_source.py` UI config

### Task: Modify Evolution Algorithm
1. Adjust parameters in `config/ui/evolution.py`
2. Modify genetic operators in `engines/generation/evolution_engine.py`
3. Change fitness calculation (multi-objective weighting)
4. Update selection method (tournament size, elitism rate)

### Task: Run Robustness Analysis
1.  Use the `ValidationOrchestrator` in `src/core/engines/validation/orchestrator.py`.
2.  Call `run_robustness_report` with a strategy AST and data.
3.  Add new tests to `src/core/engines/validation/robustness_methods/`.

### Task: Add New UI Component
1. Create new component file in `ui/src/app/components/` (e.g., `my_component.rs`)
2. Add module declaration to `ui/src/app/components/mod.rs`
3. Create render function: `pub fn render(ui: &mut egui::Ui, app: &mut MyApp)`
4. Add component to appropriate panel in `ui/src/app/mod.rs`
5. Add any new state to `ui/src/app/state.rs` and `MyApp` struct

### Task: Add New IPC Command
1. Add handler function in `src/main_api.py`
2. Add command to `COMMAND_HANDLERS` dictionary
3. Define Pydantic schema in `src/api/schemas.py` (if needed)
4. Add `AppMessage` variant in `ui/src/app/state.rs`
5. Send command from UI: `app.bridge.send_command("command_name", payload)`
6. Handle response in `MyApp.update()` match statement

### Task: Add Market Data Source
1. Upload data to Supabase Storage bucket `market-data`
2. Add metadata record to `market_data_files` table
3. Data will automatically appear in Market Data Browser
4. Use `get_ohlc_data` command to download and parse

### Task: Add ML Feature
1. Add feature calculation in `src/core/ml/features/feature_engineer.py`
2. Update feature configuration in `FeatureConfig` class
3. Feature will be automatically included in ML pipeline
4. View importance in Refine tab after training

### Task: Customize Chart Display
1. Modify rendering logic in `ui/src/app/components/ohlc_chart.rs`
2. Use `egui_plot` for additional chart types
3. Add controls in `market_data_browser.rs` for user options
4. Update `OHLCDataset` type if adding new data fields

### Task: Add Security Validation
1. Add validation function in `src/api/validation.py`
2. Call from command handler before processing
3. Raise `ValidationError`, `SecurityError`, or `ResourceLimitError`
4. Error will be caught and formatted automatically

---

## Architecture Strengths

### Design & Structure
1. **Separation of Concerns**: Clear boundaries between frontend (Rust), backend (Python), and external services (Supabase)
2. **Type Safety**:
   - Python: Runtime type checking via Polars and Pydantic
   - Rust: Compile-time type safety for UI state
3. **Semantic Awareness**: Grammar rules and metadata prevent nonsensical strategies
4. **Modularity**: Easy to add indicators, metrics, connectors, UI components

### Performance
5. **Native UI Performance**: Rust + egui provides responsive native UI with immediate mode rendering
6. **Vectorized Computation**: Polars for blazing-fast data operations (10-100x faster than pandas)
7. **Parallel Processing**: Multi-core evolution, distributed ML training
8. **IPC Efficiency**: Lightweight JSON over pipes, minimal serialization overhead

### Features & Capabilities
9. **ML Enhancement**: Meta-labeling pipeline improves strategy performance via signal filtering
10. **Cloud Integration**: Supabase for centralized market data storage and access
11. **Extensibility**: Plugin-like architecture for primitives, indicators, and ML models
12. **Code Generation**: Seamless deployment to MQL5 or Python
13. **Validation**: Multiple layers (AST structure, types, diversity, performance, robustness)
14. **Security**: Request validation, path sanitization, file size limits

### Developer Experience
15. **Two-Language Synergy**: Python for AI/ML/data, Rust for UI performance
16. **Hot Reload**: Immediate mode GUI allows rapid UI iteration
17. **Comprehensive Logging**: Detailed error messages and progress tracking
18. **Clear Architecture**: Well-documented layers and data flows

## Architecture Considerations

### Complexity
1. **Two-Language Maintenance**: Requires Python and Rust expertise for full-stack development
2. **IPC Coordination**: Debugging IPC issues requires monitoring both processes
3. **State Synchronization**: Rust UI state must stay in sync with Python backend state

### Performance Tradeoffs
4. **IPC Latency**: Small overhead from JSON serialization, though minimal for this use case
5. **Memory**: Large populations + large datasets can be memory-intensive in Python
6. **Startup Time**: Spawning Python process adds initial latency

### Dependencies & Deployment
7. **Dependency Management**: Two separate dependency ecosystems (pip + cargo)
8. **Deployment Complexity**: Need to bundle both Rust binary and Python runtime
9. **Cross-Platform**: Need to test on Windows, macOS, Linux for both Rust and Python

### Code Quality
10. **AST Coupling**: Heavy reliance on AST dictionary structure (breaking changes require widespread updates)
11. **Registry Discovery**: Reflection-based discovery may have edge cases
12. **Error Handling**: Deep call stacks in recursive AST building/execution
13. **Testing**: Integration tests must coordinate both Rust and Python

### Scaling & Future
14. **Parallelization**: Multi-processing for evolution requires careful state management
15. **Cloud Dependency**: Supabase integration creates external dependency
16. **ML Model Size**: Large ensemble models can be slow to load/save
17. **Debugging**: Generated strategies can be hard to debug (need good logging/visualization)

---

**End of Architecture Document**

## Summary for AI Agents

**TradeBias** is a hybrid Python/Rust algorithmic trading platform that:

1. **Generates** trading strategies using genetic programming with type-aware AST generation
2. **Backtests** strategies using vectorized Polars expressions for performance
3. **Refines** strategies using machine learning meta-labeling to filter signals
4. **Evaluates** performance using comprehensive metrics with dependency resolution
5. **Visualizes** results in a native Rust GUI with real-time updates via IPC
6. **Integrates** with Supabase for cloud-based market data storage and access
7. **Deploys** strategies by generating MQL5 or Python code

**Key Architecture Patterns**:
- Strategies are represented as ASTs (Abstract Syntax Trees)
- Type-driven semantic generation ensures valid strategies
- Vectorized execution via Polars for 10-100x performance
- IPC over stdin/stdout for Rust UI ↔ Python backend communication
- ML pipeline: Signal Extraction → Feature Engineering → Labeling → Training → Filtering
- Three-tier architecture: UI (Rust) → Backend (Python) → Services (Supabase)

**For Development**:
- Frontend changes: `ui/src/app/components/`
- Backend logic: `src/core/engines/`
- API commands: `src/main_api.py`
- New indicators: `src/core/functions/indicators/`
- ML features: `src/core/ml/`

**For Debugging**:
- IPC messages: Monitor stdin/stdout JSON
- Evolution progress: Watch `{"stream": "progress"}` messages
- Errors: Check `{"stream": "error"}` and Python stack traces
- UI state: Add debug logging in `MyApp.update()`

*This document provides a comprehensive reference for understanding and extending the TradeBias codebase.*
