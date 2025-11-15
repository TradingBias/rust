# TradeBias UI - Complete File Architecture

## Overview

This document provides a complete file-by-file breakdown of the proposed UI architecture, including file purposes, key types, and dependencies.

---

## Directory Structure

```
tradebias/
├── src/
│   ├── lib.rs                      # Add: pub mod ui;
│   ├── main.rs                     # NEW - Application entry point
│   │
│   ├── ui/                         # NEW MODULE - All UI code
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── state.rs
│   │   │
│   │   ├── panels/
│   │   │   ├── mod.rs
│   │   │   ├── left_panel.rs
│   │   │   ├── main_panel.rs
│   │   │   └── right_panel.rs
│   │   │
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── indicator_selector.rs
│   │   │   ├── data_selector.rs
│   │   │   ├── config_inputs.rs
│   │   │   ├── strategy_table.rs
│   │   │   └── equity_chart.rs
│   │   │
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── data_loader.rs
│   │   │   ├── evolution_runner.rs
│   │   │   └── config_bridge.rs
│   │   │
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── ui_config.rs
│   │       ├── strategy_display.rs
│   │       └── chart_data.rs
│   │
│   ├── data/
│   │   ├── mod.rs                  # MODIFY - Export connectors
│   │   ├── cache.rs                # ✅ Exists
│   │   │
│   │   └── connectors/             # NEW - Data loading
│   │       ├── mod.rs
│   │       ├── csv.rs
│   │       ├── validator.rs
│   │       └── types.rs
│   │
│   ├── types.rs                    # MODIFY - Add to_formula() methods
│   │
│   └── [existing modules...]       # ✅ engines/, functions/, config/, etc.
│
├── Cargo.toml                      # MODIFY - Add eframe, rfd
└── [existing files...]
```

---

## File Details

### Entry Point

#### `src/main.rs` (NEW)

**Purpose**: Application entry point for desktop UI

**Key Types**: None

**Dependencies**:
- `eframe`
- `tradebias::ui::TradeBiasApp`

**Code**:
```rust
use eframe::NativeOptions;
use tradebias::ui::TradeBiasApp;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Optional: for logging

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_title("TradeBias - Strategy Generator"),
        ..Default::default()
    };

    eframe::run_native(
        "TradeBias",
        options,
        Box::new(|cc| Ok(Box::new(TradeBiasApp::new(cc)))),
    )
}
```

**Lines**: ~20

---

### UI Module Root

#### `src/ui/mod.rs` (NEW)

**Purpose**: Export all UI modules

**Code**:
```rust
mod app;
mod state;
mod panels;
mod widgets;
mod services;
mod models;

pub use app::TradeBiasApp;
pub use state::AppState;
```

**Lines**: ~10

---

### Core UI Files

#### `src/ui/app.rs` (NEW)

**Purpose**: Main application struct and event loop

**Key Types**:
- `TradeBiasApp` - Main app struct implementing `eframe::App`

**Dependencies**:
- `egui`, `eframe`
- `AppState`
- `LeftPanel`, `MainPanel`, `RightPanel`

**Key Methods**:
- `new(cc: &eframe::CreationContext) -> Self`
- `update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)` - Main render loop

**Structure**:
```rust
pub struct TradeBiasApp {
    state: AppState,
    left_panel: LeftPanel,
    main_panel: MainPanel,
    right_panel: RightPanel,
}

impl eframe::App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Config").clicked() { /* ... */ }
                    if ui.button("Load Config").clicked() { /* ... */ }
                    ui.separator();
                    if ui.button("Exit").clicked() { frame.close(); }
                });
            });
        });

        egui::SidePanel::left("left_panel").min_width(300.0).show(ctx, |ui| {
            self.left_panel.show(ui, &mut self.state);
        });

        egui::SidePanel::right("right_panel").min_width(400.0).show(ctx, |ui| {
            self.right_panel.show(ui, &mut self.state);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_panel.show(ui, &mut self.state);
        });

        // Poll background tasks
        if let Some(runner) = &mut self.state.evolution_runner {
            runner.poll_progress(&mut self.state);
        }
    }
}
```

**Lines**: ~100-150

---

#### `src/ui/state.rs` (NEW)

**Purpose**: Centralized application state

**Key Types**:
- `AppState` - All UI state
- `ExecutionState` - Evolution execution status

**Dependencies**:
- `polars::DataFrame`
- `std::path::PathBuf`
- `std::collections::HashSet`
- Backend config types
- `StrategyDisplay`
- `EvolutionRunner`

**Structure**:
```rust
use std::path::PathBuf;
use std::collections::HashSet;
use polars::prelude::DataFrame;
use crate::config::{BacktestingConfig, EvolutionConfig, TradeManagementConfig};
use crate::data::connectors::DataPreview;
use super::models::StrategyDisplay;
use super::services::EvolutionRunner;

#[derive(Default)]
pub struct AppState {
    // Data
    pub data_file_path: Option<PathBuf>,
    pub loaded_data: Option<DataFrame>,
    pub data_preview: Option<DataPreview>,

    // Indicator Selection
    pub available_indicators: Vec<IndicatorInfo>,
    pub selected_indicators: HashSet<String>,

    // Trade Management
    pub initial_capital: f64,
    pub commission: f64,
    pub slippage: f64,
    pub position_sizing: PositionSizingUI,
    pub stop_loss: StopLossUI,
    pub take_profit: TakeProfitUI,
    pub max_positions: usize,

    // Evolution Config
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_count: usize,
    pub max_tree_depth: usize,
    pub tournament_size: usize,

    // Backtesting Config
    pub validation_method: ValidationMethodUI,
    pub train_test_split: f64,
    pub num_folds: usize,

    // Execution State
    pub is_running: bool,
    pub current_generation: usize,
    pub total_generations: usize,
    pub status_message: String,
    pub evolution_runner: Option<EvolutionRunner>,

    // Results
    pub hall_of_fame: Vec<StrategyDisplay>,
    pub selected_strategy_idx: Option<usize>,

    // UI State
    pub sort_column: String,
    pub sort_ascending: bool,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct IndicatorInfo {
    pub name: String,
    pub alias: String,
    pub category: String,
}

// UI-specific enums (simpler than backend enums)
pub enum PositionSizingUI {
    Fixed { size: f64 },
    Percent { percent: f64 },
    Kelly { fraction: f64 },
}

pub enum StopLossUI {
    None,
    Fixed { percent: f64 },
    ATR { multiplier: f64, period: usize },
}

pub enum TakeProfitUI {
    None,
    Fixed { percent: f64 },
    RiskReward { ratio: f64 },
}

pub enum ValidationMethodUI {
    Simple,
    WalkForward { anchored: bool },
    KFold,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            commission: 0.001,
            slippage: 0.0005,
            position_sizing: PositionSizingUI::Percent { percent: 0.02 },
            stop_loss: StopLossUI::ATR { multiplier: 2.0, period: 14 },
            take_profit: TakeProfitUI::RiskReward { ratio: 2.0 },
            max_positions: 5,
            population_size: 500,
            num_generations: 100,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            elitism_count: 10,
            max_tree_depth: 12,
            tournament_size: 7,
            validation_method: ValidationMethodUI::WalkForward { anchored: false },
            train_test_split: 0.7,
            num_folds: 5,
            sort_column: "rank".to_string(),
            sort_ascending: true,
            available_indicators: Vec::new(),
            selected_indicators: HashSet::new(),
            data_file_path: None,
            loaded_data: None,
            data_preview: None,
            is_running: false,
            current_generation: 0,
            total_generations: 0,
            status_message: "Ready".to_string(),
            evolution_runner: None,
            hall_of_fame: Vec::new(),
            selected_strategy_idx: None,
            error_message: None,
        }
    }
}
```

**Lines**: ~200-250

---

### Panels

#### `src/ui/panels/mod.rs` (NEW)

```rust
mod left_panel;
mod main_panel;
mod right_panel;

pub use left_panel::LeftPanel;
pub use main_panel::MainPanel;
pub use right_panel::RightPanel;
```

#### `src/ui/panels/left_panel.rs` (NEW)

**Purpose**: Configuration inputs panel

**Key Types**:
- `LeftPanel` - Panel struct

**Dependencies**:
- `AppState`
- `DataSelector`, `IndicatorSelector`, `ConfigInputs` widgets

**Structure**:
```rust
use egui;
use super::super::state::AppState;
use super::super::widgets::{DataSelector, IndicatorSelector, ConfigInputs};

pub struct LeftPanel {
    data_selector: DataSelector,
    indicator_selector: IndicatorSelector,
    config_inputs: ConfigInputs,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            data_selector: DataSelector::new(),
            indicator_selector: IndicatorSelector::new(),
            config_inputs: ConfigInputs::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Configuration");
            ui.separator();

            // Data Section
            ui.collapsing("1. Data", |ui| {
                self.data_selector.show(ui, state);
            });

            // Indicators Section
            ui.collapsing("2. Indicators", |ui| {
                self.indicator_selector.show(ui, state);
            });

            // Trade Management Section
            ui.collapsing("3. Trade Management", |ui| {
                self.config_inputs.show_trade_management(ui, state);
            });

            // Evolution & Backtesting Section
            ui.collapsing("4. Evolution & Backtesting", |ui| {
                self.config_inputs.show_evolution(ui, state);
                ui.separator();
                self.config_inputs.show_backtesting(ui, state);
            });

            ui.separator();

            // Control Buttons
            ui.horizontal(|ui| {
                if ui.add_enabled(!state.is_running, egui::Button::new("▶ Run Evolution")).clicked() {
                    self.on_run_clicked(state);
                }
                if ui.add_enabled(state.is_running, egui::Button::new("⏹ Stop")).clicked() {
                    self.on_stop_clicked(state);
                }
            });

            if ui.button("Reset Config").clicked() {
                *state = AppState::default();
            }
        });
    }

    fn on_run_clicked(&mut self, state: &mut AppState) {
        // Validate and start evolution
        // See services/evolution_runner.rs
    }

    fn on_stop_clicked(&mut self, state: &mut AppState) {
        if let Some(runner) = &mut state.evolution_runner {
            runner.cancel();
        }
        state.is_running = false;
    }
}
```

**Lines**: ~150-200

#### `src/ui/panels/main_panel.rs` (NEW)

**Purpose**: Results table panel

**Key Types**:
- `MainPanel`

**Dependencies**:
- `AppState`
- `StrategyTable` widget

**Lines**: ~100-150

#### `src/ui/panels/right_panel.rs` (NEW)

**Purpose**: Equity chart and strategy details panel

**Key Types**:
- `RightPanel`

**Dependencies**:
- `AppState`
- `EquityChart` widget

**Lines**: ~150-200

---

### Widgets

#### `src/ui/widgets/mod.rs` (NEW)

```rust
mod indicator_selector;
mod data_selector;
mod config_inputs;
mod strategy_table;
mod equity_chart;

pub use indicator_selector::IndicatorSelector;
pub use data_selector::DataSelector;
pub use config_inputs::ConfigInputs;
pub use strategy_table::StrategyTable;
pub use equity_chart::EquityChart;
```

#### `src/ui/widgets/indicator_selector.rs` (NEW)

**Lines**: ~100-150

#### `src/ui/widgets/data_selector.rs` (NEW)

**Lines**: ~80-120

#### `src/ui/widgets/config_inputs.rs` (NEW)

**Lines**: ~200-300 (many input fields)

#### `src/ui/widgets/strategy_table.rs` (NEW)

**Lines**: ~150-200

#### `src/ui/widgets/equity_chart.rs` (NEW)

**Lines**: ~100-150

---

### Services

#### `src/ui/services/mod.rs` (NEW)

```rust
mod data_loader;
mod evolution_runner;
mod config_bridge;

pub use data_loader::DataLoader;
pub use evolution_runner::EvolutionRunner;
pub use config_bridge::ConfigBridge;
```

#### `src/ui/services/data_loader.rs` (NEW)

**Purpose**: Load and validate CSV files

**Dependencies**:
- `crate::data::connectors::CsvConnector`

**Lines**: ~50-80

#### `src/ui/services/evolution_runner.rs` (NEW)

**Purpose**: Run evolution in background thread

**Dependencies**:
- `std::thread`
- `std::sync::mpsc::{channel, Sender, Receiver}`
- `crate::engines::generation::EvolutionEngine`

**Lines**: ~150-200

#### `src/ui/services/config_bridge.rs` (NEW)

**Purpose**: Convert UI config to backend config

**Lines**: ~100-150

---

### Models

#### `src/ui/models/mod.rs` (NEW)

```rust
mod ui_config;
mod strategy_display;
mod chart_data;

pub use ui_config::*;
pub use strategy_display::StrategyDisplay;
pub use chart_data::ChartData;
```

#### `src/ui/models/strategy_display.rs` (NEW)

**Lines**: ~80-120

---

### Backend Extensions

#### `src/data/connectors/mod.rs` (NEW)

**Purpose**: Export CSV connector

**Code**:
```rust
mod csv;
mod validator;
mod types;

pub use csv::CsvConnector;
pub use types::{DataPreview, DatasetMetadata, ColumnStats, RequiredColumn};
pub use validator::DataValidator;
```

#### `src/data/connectors/csv.rs` (NEW)

**Lines**: ~200-300

#### `src/data/connectors/validator.rs` (NEW)

**Lines**: ~150-200

#### `src/data/connectors/types.rs` (NEW)

**Lines**: ~80-120

---

#### `src/types.rs` (MODIFY)

**Changes**: Add pretty printer methods to `AstNode`

**Add**:
```rust
impl AstNode {
    pub fn to_formula(&self) -> String {
        // See 02-ast-pretty-printer.md
    }

    pub fn to_formula_truncated(&self, max_len: usize) -> String {
        let full = self.to_formula();
        if full.len() > max_len {
            format!("{}...", &full[..max_len - 3])
        } else {
            full
        }
    }
}
```

**Lines Added**: ~30-50

---

#### `src/data/mod.rs` (MODIFY)

**Changes**: Export connectors module

**Before**:
```rust
pub mod cache;

pub use cache::IndicatorCache;
```

**After**:
```rust
pub mod cache;
pub mod connectors;

pub use cache::IndicatorCache;
pub use connectors::{CsvConnector, DataPreview, DatasetMetadata};
```

---

#### `src/lib.rs` (MODIFY)

**Changes**: Add UI module export

**Add**:
```rust
pub mod ui;
```

---

## Total Line Count Estimates

### New Files

| Module | Lines |
|--------|-------|
| **UI Core** | |
| `main.rs` | 20 |
| `ui/mod.rs` | 10 |
| `ui/app.rs` | 150 |
| `ui/state.rs` | 250 |
| **Panels** | |
| `ui/panels/mod.rs` | 10 |
| `ui/panels/left_panel.rs` | 200 |
| `ui/panels/main_panel.rs` | 150 |
| `ui/panels/right_panel.rs` | 200 |
| **Widgets** | |
| `ui/widgets/mod.rs` | 10 |
| `ui/widgets/indicator_selector.rs` | 120 |
| `ui/widgets/data_selector.rs` | 100 |
| `ui/widgets/config_inputs.rs` | 300 |
| `ui/widgets/strategy_table.rs` | 180 |
| `ui/widgets/equity_chart.rs` | 120 |
| **Services** | |
| `ui/services/mod.rs` | 10 |
| `ui/services/data_loader.rs` | 70 |
| `ui/services/evolution_runner.rs` | 180 |
| `ui/services/config_bridge.rs` | 120 |
| **Models** | |
| `ui/models/mod.rs` | 10 |
| `ui/models/strategy_display.rs` | 100 |
| `ui/models/ui_config.rs` | 50 |
| `ui/models/chart_data.rs` | 40 |
| **Data Connectors** | |
| `data/connectors/mod.rs` | 10 |
| `data/connectors/csv.rs` | 280 |
| `data/connectors/validator.rs` | 180 |
| `data/connectors/types.rs` | 100 |
| **Total New Lines** | **~2,940** |

### Modified Files

| File | Lines Added |
|------|-------------|
| `src/lib.rs` | 1 |
| `src/data/mod.rs` | 3 |
| `src/types.rs` | 40 |
| **Total Modified Lines** | **~44** |

### Grand Total

**New Code**: ~2,984 lines
**New Files**: 27
**Modified Files**: 3

---

## Dependencies Graph

```
main.rs
  └─> ui::TradeBiasApp
        ├─> ui::AppState
        │     ├─> config::* (backend)
        │     ├─> data::connectors::DataPreview
        │     └─> ui::models::StrategyDisplay
        │
        ├─> ui::panels::LeftPanel
        │     ├─> ui::widgets::DataSelector
        │     │     └─> ui::services::DataLoader
        │     │           └─> data::connectors::CsvConnector
        │     │
        │     ├─> ui::widgets::IndicatorSelector
        │     │     └─> functions::registry::FunctionRegistry (backend)
        │     │
        │     └─> ui::widgets::ConfigInputs
        │
        ├─> ui::panels::MainPanel
        │     └─> ui::widgets::StrategyTable
        │
        └─> ui::panels::RightPanel
              └─> ui::widgets::EquityChart
                    └─> egui::plot
```

---

## Summary

This architecture provides:

- ✅ **Clean Separation**: UI, services, and backend are decoupled
- ✅ **Modularity**: Each widget/panel is self-contained
- ✅ **Testability**: Services can be tested independently
- ✅ **Maintainability**: Clear structure, easy to find code
- ✅ **Scalability**: Easy to add new panels/widgets

**Total Effort**: ~3,000 lines of new code across 27 files, plus minor modifications to 3 existing files.
