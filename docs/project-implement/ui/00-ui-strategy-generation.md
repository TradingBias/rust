# UI Strategy Generation Implementation Plan

## Overview

This document outlines the implementation of a graphical user interface (GUI) for automated strategy generation using the existing TradeBias evolution engine. The UI will provide an intuitive three-panel layout for configuring, executing, and visualizing strategy evolution results.

### Target Architecture
- **Framework**: egui (already in dependencies)
- **Rendering**: Native desktop application
- **State Management**: Centralized application state with message passing
- **Module Design**: Clean separation between UI, business logic, and backend integration

---

## UI Layout Design

```
┌─────────────────────────────────────────────────────────────────┐
│ TradeBias - Strategy Generator                          [_][□][X]│
├──────────────┬───────────────────────────────┬──────────────────┤
│              │                               │                  │
│              │                               │                  │
│  Left Panel  │      Main Panel               │   Right Panel    │
│              │                               │                  │
│  Options &   │      Results Table            │   Equity Chart   │
│  Config      │                               │                  │
│              │                               │                  │
│  - Data      │  [Strategy List with Metrics] │   [Line Chart]   │
│  - Indicators│                               │                  │
│  - Trade Cfg │                               │                  │
│  - Evolution │                               │                  │
│              │                               │                  │
│  [Run]       │  [Sort/Filter Controls]       │   [Metrics]      │
│  [Stop]      │                               │                  │
│              │                               │                  │
└──────────────┴───────────────────────────────┴──────────────────┘
```

---

## File Architecture

### Proposed Structure

```
src/
├── ui/
│   ├── mod.rs                      # Main UI module exports
│   ├── app.rs                      # Main application state & loop
│   ├── state.rs                    # Application state management
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── left_panel.rs           # Configuration panel
│   │   ├── main_panel.rs           # Results table panel
│   │   └── right_panel.rs          # Equity chart panel
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── indicator_selector.rs   # Multi-select indicator widget
│   │   ├── data_selector.rs        # File picker widget
│   │   ├── config_inputs.rs        # Numeric input widgets
│   │   ├── strategy_table.rs       # Sortable results table
│   │   └── equity_chart.rs         # Line chart for equity curves
│   ├── models/
│   │   ├── mod.rs
│   │   ├── ui_config.rs            # UI-specific config models
│   │   ├── strategy_display.rs     # Display models for strategies
│   │   └── chart_data.rs           # Chart data structures
│   └── services/
│       ├── mod.rs
│       ├── data_loader.rs          # CSV loading service
│       ├── evolution_runner.rs     # Async evolution execution
│       └── config_bridge.rs        # Bridge UI config to backend config
│
├── lib.rs                          # Add pub mod ui
└── main.rs                         # Entry point (NEW)
```

---

## Module Breakdown

### 1. Main Application (`ui/app.rs`)

**Responsibilities**:
- Initialize egui application
- Manage three-panel layout
- Route events between panels
- Handle application lifecycle

**Key Structure**:
```rust
pub struct TradeBiasApp {
    state: AppState,
    left_panel: LeftPanel,
    main_panel: MainPanel,
    right_panel: RightPanel,
}

impl eframe::App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Render three-panel layout
    }
}
```

---

### 2. Application State (`ui/state.rs`)

**Responsibilities**:
- Central source of truth for all UI state
- Manages evolution execution state
- Holds configuration values
- Stores results and selected strategy

**Key Structure**:
```rust
pub struct AppState {
    // Data Configuration
    pub data_file_path: Option<PathBuf>,
    pub loaded_data: Option<DataFrame>,
    pub data_preview: Option<DataPreview>,

    // Indicator Selection
    pub available_indicators: Vec<IndicatorInfo>,
    pub selected_indicators: HashSet<String>,

    // Trade Management Configuration
    pub initial_capital: f64,
    pub commission: f64,
    pub slippage: f64,
    pub stop_loss: StopLossConfig,
    pub take_profit: TakeProfitConfig,
    pub position_sizing: PositionSizing,
    pub max_positions: usize,

    // Evolution Configuration
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_count: usize,
    pub max_tree_depth: usize,
    pub tournament_size: usize,

    // Backtesting Configuration
    pub validation_method: ValidationMethod,
    pub train_test_split: f64,
    pub num_folds: usize,

    // Execution State
    pub is_running: bool,
    pub current_generation: usize,
    pub progress_percentage: f32,
    pub status_message: String,

    // Results
    pub hall_of_fame: Vec<StrategyDisplay>,
    pub selected_strategy_idx: Option<usize>,

    // Sorting/Filtering
    pub sort_column: String,
    pub sort_ascending: bool,
    pub filter_min_trades: Option<usize>,
}
```

---

### 3. Left Panel (`ui/panels/left_panel.rs`)

**Responsibilities**:
- Display configuration sections (collapsible)
- Capture user inputs
- Trigger evolution execution
- Validate configuration

**Sections**:

#### 3.1 Data Selection
```rust
// Widget: File picker button
// Shows: Selected file path, row count, column preview
// Validation: Must have OHLCV columns
```

#### 3.2 Indicator Selection
```rust
// Widget: Multi-select checkbox list
// Grouped by category: Trend, Momentum, Volatility, Volume
// Shows: Indicator name, default parameters
```

#### 3.3 Trade Management
```rust
// Inputs:
// - Initial Capital: f64 (default: 10000.0)
// - Commission: f64 (default: 0.001 = 0.1%)
// - Slippage: f64 (default: 0.0005 = 0.05%)
// - Position Sizing: Enum dropdown
//   - Fixed (size in $)
//   - Percent (% of capital)
//   - Kelly (fraction)
// - Max Positions: usize (default: 5)
// - Stop Loss: Enum dropdown
//   - None
//   - Fixed % (slider)
//   - ATR (multiplier + period)
// - Take Profit: Enum dropdown
//   - None
//   - Fixed % (slider)
//   - Risk/Reward ratio (slider)
```

#### 3.4 Evolution & Backtesting
```rust
// Evolution params:
// - Population Size: 50-5000 (default: 500)
// - Generations: 10-1000 (default: 100)
// - Mutation Rate: 0.0-1.0 (default: 0.15)
// - Crossover Rate: 0.0-1.0 (default: 0.85)
// - Elitism Count: 1-50 (default: 10)
// - Max Tree Depth: 3-20 (default: 12)
// - Tournament Size: 2-20 (default: 7)

// Backtesting params:
// - Validation Method: Enum
//   - Simple Split
//   - Walk-Forward (Anchored/Rolling)
//   - K-Fold
// - Train/Test Split: 0.5-0.9 (default: 0.7)
// - Num Folds: 2-10 (default: 5) [if K-Fold]
```

#### 3.5 Control Buttons
```rust
// [Run Evolution] - Primary button, disabled if validation fails
// [Stop] - Enabled only during execution
// [Reset Config] - Reset to defaults
```

---

### 4. Main Panel (`ui/panels/main_panel.rs`)

**Responsibilities**:
- Display Hall of Fame strategies in a table
- Support sorting by any column
- Handle row selection
- Show progress bar during execution

**Table Columns**:
1. Rank (1-N)
2. Fitness Score
3. Return %
4. Total Trades
5. Win Rate %
6. Max Drawdown %
7. Sharpe Ratio
8. Formula (truncated, tooltip shows full)

**Features**:
- Click column header to sort
- Click row to select (highlights)
- Progress bar at top during evolution
- Status message display
- Export button (CSV export of strategies)

---

### 5. Right Panel (`ui/panels/right_panel.rs`)

**Responsibilities**:
- Display equity curve for selected strategy
- Show key metrics in a card layout
- Provide trade-by-trade details (expandable)

**Content When No Selection**:
```
[Empty state]
"Select a strategy from the results to view details"
```

**Content When Strategy Selected**:

#### 5.1 Metrics Card
```
┌─────────────────────────────┐
│ Strategy Metrics            │
├─────────────────────────────┤
│ Return: +45.3%              │
│ Sharpe: 1.82                │
│ Max DD: -12.4%              │
│ Win Rate: 58.2%             │
│ Total Trades: 124           │
│ Avg Trade: +0.36%           │
└─────────────────────────────┘
```

#### 5.2 Equity Chart
- X-axis: Bar index or date
- Y-axis: Portfolio value
- Line plot with grid
- Hover tooltip showing exact values

#### 5.3 Formula Display
```
┌─────────────────────────────┐
│ Strategy Formula            │
├─────────────────────────────┤
│ AND(                        │
│   >(RSI(14), 30),          │
│   <(RSI(14), 70),          │
│   >(EMA(12), SMA(26))      │
│ )                           │
└─────────────────────────────┘
```

#### 5.4 Trade List (Collapsible)
- Scrollable table of all trades
- Columns: Entry Bar, Exit Bar, P&L, Direction, Exit Reason

---

## Services Layer

### 6.1 Data Loader (`ui/services/data_loader.rs`)

**Responsibilities**:
- Load CSV files using Polars
- Validate required columns (OHLCV)
- Generate data preview
- Handle errors gracefully

**Interface**:
```rust
pub struct DataLoader;

impl DataLoader {
    pub fn load_csv(path: &Path) -> Result<DataFrame, DataError> {
        // Use polars::prelude::CsvReader
    }

    pub fn validate_ohlcv(df: &DataFrame) -> Result<(), DataError> {
        // Check for required columns: open, high, low, close, volume
    }

    pub fn create_preview(df: &DataFrame) -> DataPreview {
        // First 10 rows, column stats
    }
}
```

---

### 6.2 Evolution Runner (`ui/services/evolution_runner.rs`)

**Responsibilities**:
- Convert UI config to backend config structs
- Run evolution in background thread
- Report progress via channel
- Handle cancellation

**Interface**:
```rust
pub struct EvolutionRunner {
    handle: Option<JoinHandle<Result<Vec<EliteStrategy>>>>,
    progress_rx: Receiver<ProgressUpdate>,
    cancel_tx: Sender<()>,
}

impl EvolutionRunner {
    pub fn start(
        config: EvolutionRunConfig,
        data: DataFrame,
    ) -> Self {
        // Spawn thread, setup channels
    }

    pub fn poll_progress(&mut self) -> Option<ProgressUpdate> {
        // Non-blocking check for progress updates
    }

    pub fn cancel(&mut self) {
        // Send cancellation signal
    }

    pub fn try_get_results(&mut self) -> Option<Result<Vec<EliteStrategy>>> {
        // Check if complete, return results
    }
}

pub struct ProgressUpdate {
    pub generation: usize,
    pub total_generations: usize,
    pub best_fitness: f64,
    pub hall_size: usize,
}
```

---

### 6.3 Config Bridge (`ui/services/config_bridge.rs`)

**Responsibilities**:
- Convert UI state to backend config types
- Ensure type safety
- Apply validation

**Interface**:
```rust
pub struct ConfigBridge;

impl ConfigBridge {
    pub fn to_backtesting_config(state: &AppState) -> BacktestingConfig {
        BacktestingConfig {
            validation_method: state.validation_method.clone(),
            train_test_split: state.train_test_split,
            num_folds: state.num_folds,
            initial_capital: state.initial_capital,
            commission: state.commission,
            slippage: state.slippage,
        }
    }

    pub fn to_evolution_config(state: &AppState) -> crate::config::EvolutionConfig {
        // Map UI state to backend EvolutionConfig
    }

    pub fn to_trade_management_config(state: &AppState) -> TradeManagementConfig {
        // Map UI state to backend TradeManagementConfig
    }
}
```

---

## Widgets

### 7.1 Indicator Selector (`ui/widgets/indicator_selector.rs`)

**Interface**:
```rust
pub struct IndicatorSelector;

impl IndicatorSelector {
    pub fn show(
        ui: &mut egui::Ui,
        indicators: &[IndicatorInfo],
        selected: &mut HashSet<String>,
    ) {
        // Render grouped checkboxes
        // Category headers: Trend, Momentum, Volatility, Volume
    }
}

#[derive(Clone)]
pub struct IndicatorInfo {
    pub name: String,
    pub alias: String,
    pub category: IndicatorCategory,
    pub description: String,
}

pub enum IndicatorCategory {
    Trend,
    Momentum,
    Volatility,
    Volume,
}
```

**Data Source**: Query `FunctionRegistry::get_indicators()` on startup

---

### 7.2 Data Selector (`ui/widgets/data_selector.rs`)

**Interface**:
```rust
pub struct DataSelector;

impl DataSelector {
    pub fn show(
        ui: &mut egui::Ui,
        current_path: &mut Option<PathBuf>,
        data: &mut Option<DataFrame>,
    ) -> bool {
        // Returns true if new file loaded
        // Shows file dialog on button click
        // Displays current file info
    }
}
```

**Dependencies**: Use `rfd` crate for native file dialogs (add to Cargo.toml)

---

### 7.3 Strategy Table (`ui/widgets/strategy_table.rs`)

**Interface**:
```rust
pub struct StrategyTable;

impl StrategyTable {
    pub fn show(
        ui: &mut egui::Ui,
        strategies: &[StrategyDisplay],
        selected_idx: &mut Option<usize>,
        sort_column: &mut String,
        sort_ascending: &mut bool,
    ) {
        // Render table with sortable columns
        // Handle row selection
    }
}
```

---

### 7.4 Equity Chart (`ui/widgets/equity_chart.rs`)

**Interface**:
```rust
pub struct EquityChart;

impl EquityChart {
    pub fn show(
        ui: &mut egui::Ui,
        equity_curve: &[f64],
        initial_capital: f64,
    ) {
        // Render line chart using egui::plot
    }
}
```

**Dependencies**: Use `egui::plot` module (built-in)

---

## Models

### 8.1 UI Config (`ui/models/ui_config.rs`)

Re-export backend config types for UI convenience, or create UI-specific wrappers if needed.

---

### 8.2 Strategy Display (`ui/models/strategy_display.rs`)

**Interface**:
```rust
#[derive(Clone)]
pub struct StrategyDisplay {
    pub rank: usize,
    pub fitness: f64,
    pub return_pct: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub formula: String,          // Human-readable AST
    pub equity_curve: Vec<f64>,
    pub trades: Vec<Trade>,
}

impl From<EliteStrategy> for StrategyDisplay {
    fn from(elite: EliteStrategy) -> Self {
        // Convert backend EliteStrategy to display model
    }
}
```

---

## Required Missing Modules

### Missing Backend Functionality

Based on code review, the following backend modules need implementation or verification:

1. **CSV Data Connector** (`src/data/connectors/`)
   - Current status: Stub file exists
   - Needed: CSV loading with Polars, OHLCV validation

2. **Formula Pretty Printing**
   - Current status: AST exists but no human-readable formatter
   - Needed: Convert `AstNode` to readable string (e.g., "AND(>(RSI(14), 30))")

3. **Elite Strategy Structure**
   - Current status: Exists in `hall_of_fame.rs`
   - Verify: Contains all needed fields (equity curve, trades, metrics)

4. **Progress Callback Implementation**
   - Current status: Trait defined in `evolution_engine.rs`
   - Needed: Channel-based implementation for UI updates

---

## Additional Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
eframe = "0.32.3"       # egui framework (already have egui)
rfd = "0.15"            # Native file dialogs
tokio = { version = "1", features = ["rt-multi-thread", "sync"] }  # Async runtime
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create UI module structure
- [ ] Implement AppState
- [ ] Implement DataLoader service
- [ ] Implement CSV connector backend module
- [ ] Create basic three-panel layout (empty)

### Phase 2: Left Panel (Week 2)
- [ ] Implement data selector widget
- [ ] Implement indicator selector widget
- [ ] Create trade management input form
- [ ] Create evolution config input form
- [ ] Add validation logic

### Phase 3: Main Panel (Week 3)
- [ ] Implement strategy table widget
- [ ] Add sorting functionality
- [ ] Implement EvolutionRunner service
- [ ] Add progress bar display
- [ ] Wire up Run/Stop buttons

### Phase 4: Right Panel (Week 4)
- [ ] Implement equity chart widget
- [ ] Create metrics card display
- [ ] Add formula pretty printer
- [ ] Implement trade list display
- [ ] Add export functionality

### Phase 5: Integration & Polish (Week 5)
- [ ] Connect all panels via AppState
- [ ] Add error handling and user feedback
- [ ] Implement config persistence (save/load)
- [ ] Add keyboard shortcuts
- [ ] Performance optimization
- [ ] User testing and refinement

---

## Testing Strategy

### Unit Tests
- DataLoader: CSV parsing, validation
- ConfigBridge: Conversion accuracy
- StrategyDisplay: Model conversion

### Integration Tests
- Full evolution run from UI
- Config -> Backend -> Results flow
- Progress updates during execution

### UI Tests
- Manual testing checklist
- Verify all inputs accept valid ranges
- Confirm proper error messages
- Test responsiveness during long runs

---

## Future Enhancements (Post-MVP)

1. **Multi-Threading**: Parallel strategy evaluation
2. **Real-time Chart Updates**: Show equity curve during evolution
3. **Strategy Comparison**: Select multiple strategies to compare
4. **Export Options**: Export strategies as code, JSON config
5. **Preset Configs**: Save/load common configurations
6. **Dark Mode**: Theme support
7. **Advanced Filters**: Filter Hall of Fame by multiple criteria
8. **Walk-Forward Chart**: Visualize out-of-sample performance
9. **Optimization Grid**: Parameter sweep visualization

---

## Notes & Considerations

### Performance
- Use `Arc` for sharing data between UI and evolution thread
- Consider result pagination if Hall of Fame > 1000 strategies
- Lazy-load equity curves (only for selected strategy)

### UX
- Show validation errors inline (red text, icons)
- Disable incompatible options (e.g., num_folds when not using K-Fold)
- Provide tooltips with parameter explanations
- Save last-used config on exit

### Error Handling
- Graceful degradation: Show partial results if evolution crashes mid-run
- User-friendly error messages: "Could not load CSV: missing 'close' column"
- Log errors to file for debugging

---

## Summary

This implementation plan provides a comprehensive roadmap for building the TradeBias UI strategy generation interface. The modular architecture ensures:

- **Separation of Concerns**: UI, business logic, and backend are decoupled
- **Testability**: Each module can be tested independently
- **Maintainability**: Clear structure makes future enhancements easy
- **User Experience**: Three-panel design provides intuitive workflow

All backend modules referenced in this plan already exist in the codebase except for the CSV connector, which requires implementation.
