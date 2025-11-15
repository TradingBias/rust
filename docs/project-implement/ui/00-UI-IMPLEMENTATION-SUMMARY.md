# TradeBias UI Implementation - Quick Start Summary

## Overview

This is the **quick reference guide** for implementing the TradeBias strategy generation UI. For detailed specifications, see the individual documents listed below.

---

## Document Overview

| Document | Purpose | Priority | Estimated Time |
|----------|---------|----------|----------------|
| [00-ui-strategy-generation.md](00-ui-strategy-generation.md) | Complete UI architecture & implementation plan | **CRITICAL** | 5 weeks |
| [01-csv-data-connector.md](01-csv-data-connector.md) | CSV loading module (backend dependency) | **HIGH** | 4-6 hours |
| [02-ast-pretty-printer.md](02-ast-pretty-printer.md) | Formula display formatter (backend dependency) | **HIGH** | 2-4 hours |

---

## Architecture at a Glance

### Three-Panel Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ TradeBias - Strategy Generator                                  │
├──────────────┬───────────────────────────────┬──────────────────┤
│              │                               │                  │
│  LEFT PANEL  │      MAIN PANEL               │   RIGHT PANEL    │
│              │                               │                  │
│  Config:     │  Results:                     │   Chart:         │
│  • CSV Data  │  • Hall of Fame Table         │   • Equity Curve │
│  • Indicators│  • Sortable Columns           │   • Metrics Card │
│  • Trade Mgmt│  • Progress Bar               │   • Trade List   │
│  • Evolution │                               │   • Formula      │
│              │                               │                  │
│  [Run]       │                               │                  │
│  [Stop]      │                               │                  │
└──────────────┴───────────────────────────────┴──────────────────┘
```

### Module Structure

```
src/ui/
├── app.rs              # Main application & event loop
├── state.rs            # Centralized application state
├── panels/
│   ├── left_panel.rs   # Configuration inputs
│   ├── main_panel.rs   # Results table
│   └── right_panel.rs  # Equity chart & details
├── widgets/
│   ├── indicator_selector.rs
│   ├── data_selector.rs
│   ├── strategy_table.rs
│   └── equity_chart.rs
├── services/
│   ├── data_loader.rs      # CSV → DataFrame
│   ├── evolution_runner.rs # Background execution
│   └── config_bridge.rs    # UI → Backend config
└── models/
    ├── ui_config.rs
    ├── strategy_display.rs
    └── chart_data.rs
```

---

## Prerequisites Checklist

### ✅ Dependencies in Cargo.toml

Add these to `[dependencies]`:

```toml
eframe = "0.32.3"      # egui framework (MISSING - ADD THIS)
rfd = "0.15"           # File dialogs (MISSING - ADD THIS)
```

Already present:
- ✅ `egui = "0.32.3"`
- ✅ `polars = "0.51.0"`
- ✅ `serde`, `serde_json`, `toml`

### ⚠️ Missing Backend Modules

**These MUST be implemented before UI can work:**

1. **CSV Data Connector** (`src/data/connectors/`)
   - File: See `01-csv-data-connector.md`
   - Status: ❌ Not implemented (stub exists)
   - Priority: **BLOCKING**

2. **AST Pretty Printer** (`src/types.rs` or `src/engines/generation/formatter.rs`)
   - File: See `02-ast-pretty-printer.md`
   - Status: ❌ Not implemented
   - Priority: **HIGH** (needed for display)

### ✅ Existing Backend Modules (Ready to Use)

- ✅ Evolution Engine (`src/engines/generation/evolution_engine.rs`)
- ✅ Backtester (`src/engines/evaluation/backtester.rs`)
- ✅ Function Registry (`src/functions/registry.rs`)
- ✅ All Config Types (`src/config/`)
- ✅ Indicator Cache (`src/data/cache.rs`)
- ✅ Core Types (`src/types.rs`)

---

## Implementation Phases

### Phase 0: Backend Prerequisites (1-2 days)

**Goal**: Implement missing backend modules

- [ ] Add `eframe` and `rfd` to `Cargo.toml`
- [ ] Implement CSV Data Connector (4-6 hours)
  - `src/data/connectors/mod.rs`
  - `src/data/connectors/csv.rs`
  - `src/data/connectors/validator.rs`
  - `src/data/connectors/types.rs`
- [ ] Implement AST Pretty Printer (2-4 hours)
  - Add `to_formula()` method to `AstNode` in `src/types.rs`
  - Add `to_formula_truncated()` helper
- [ ] Verify with unit tests

**Deliverable**: CSV loading works, formulas display correctly

---

### Phase 1: Foundation (Week 1)

**Goal**: Basic UI structure with no functionality

- [ ] Create `src/ui/` module structure
- [ ] Implement `AppState` in `state.rs`
- [ ] Create `TradeBiasApp` in `app.rs` with empty panels
- [ ] Implement three-panel layout (no content)
- [ ] Add `src/main.rs` entry point
- [ ] Verify app launches and displays empty UI

**Deliverable**: App window opens with three empty panels

---

### Phase 2: Left Panel (Week 2)

**Goal**: Configuration inputs functional

- [ ] Implement `DataSelector` widget
  - File picker button
  - Display selected file info
  - Preview first 10 rows
- [ ] Implement `IndicatorSelector` widget
  - Multi-select checkboxes
  - Group by category (Trend, Momentum, etc.)
- [ ] Implement trade management inputs
  - Initial capital, fees, slippage
  - Position sizing dropdown
  - SL/TP config
- [ ] Implement evolution config inputs
  - Population size, generations, etc.
  - Validation method dropdown
- [ ] Add validation logic
- [ ] Wire up "Run" button (no-op for now)

**Deliverable**: All config inputs capture user settings in `AppState`

---

### Phase 3: Main Panel (Week 3)

**Goal**: Evolution execution and results display

- [ ] Implement `EvolutionRunner` service
  - Spawn background thread
  - Send progress updates via channel
  - Handle cancellation
- [ ] Implement `StrategyTable` widget
  - Display Hall of Fame in table
  - Sortable columns (click header)
  - Row selection
- [ ] Add progress bar during execution
- [ ] Wire up "Run" button to trigger evolution
- [ ] Wire up "Stop" button to cancel
- [ ] Display results when complete

**Deliverable**: Can run evolution and see results in table

---

### Phase 4: Right Panel (Week 4)

**Goal**: Strategy details and visualization

- [ ] Implement `EquityChart` widget using `egui::plot`
- [ ] Display selected strategy metrics
- [ ] Show pretty-printed formula
- [ ] Add trade list (collapsible)
- [ ] Add export button (CSV)

**Deliverable**: Clicking a strategy shows full details

---

### Phase 5: Polish (Week 5)

**Goal**: Production-ready UI

- [ ] Error handling with user-friendly messages
- [ ] Config persistence (save/load to TOML)
- [ ] Keyboard shortcuts (Ctrl+R to run, Esc to stop)
- [ ] Input validation with inline errors
- [ ] Performance optimization (large datasets)
- [ ] User testing and refinement

**Deliverable**: Polished, user-friendly application

---

## Quick Implementation Snippets

### Main Entry Point (`src/main.rs`)

```rust
use eframe::NativeOptions;
use tradebias::ui::TradeBiasApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "TradeBias",
        NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400.0, 800.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(TradeBiasApp::new(cc)))),
    )
}
```

### Basic App Structure (`src/ui/app.rs`)

```rust
pub struct TradeBiasApp {
    state: AppState,
}

impl TradeBiasApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Three panels here
            });
        });
    }
}
```

### Loading CSV Data (`ui/services/data_loader.rs`)

```rust
use crate::data::connectors::CsvConnector;

pub fn load_csv(path: &Path) -> Result<(DataFrame, DataPreview), String> {
    let (df, _) = CsvConnector::load_and_validate(path, None)
        .map_err(|e| e.to_string())?;
    let preview = CsvConnector::create_preview(path, &df)
        .map_err(|e| e.to_string())?;
    let normalized = CsvConnector::normalize_columns(df)
        .map_err(|e| e.to_string())?;
    Ok((normalized, preview))
}
```

---

## Testing Checklist

### Unit Tests
- [ ] CSV Connector: Valid data, invalid data, missing columns
- [ ] AST Formatter: Nested expressions, truncation
- [ ] Config Bridge: UI config → Backend config conversion

### Integration Tests
- [ ] Load CSV → Run evolution → Display results (end-to-end)
- [ ] Save/load configuration persistence
- [ ] Large dataset handling (10k+ rows)

### Manual UI Tests
- [ ] Load various CSV formats
- [ ] Select/deselect indicators
- [ ] Modify all config inputs
- [ ] Run evolution and verify progress
- [ ] Sort table by each column
- [ ] Select strategy and view chart
- [ ] Export results

---

## Common Issues & Solutions

### Issue: "Cannot find module `ui` in crate root"
**Solution**: Add `pub mod ui;` to `src/lib.rs`

### Issue: "No method named `to_formula` on type `AstNode`"
**Solution**: Implement AST Pretty Printer (see doc 02)

### Issue: "Cannot find `CsvConnector` in module `data::connectors`"
**Solution**: Implement CSV Data Connector (see doc 01)

### Issue: UI freezes during evolution
**Solution**: Ensure evolution runs in background thread, not on UI thread

### Issue: Large CSV files cause UI lag
**Solution**: Use `Arc<DataFrame>` and load in background thread

---

## Success Criteria

### Minimum Viable Product (MVP)
- ✅ User can load CSV file
- ✅ User can select indicators
- ✅ User can configure basic trade settings
- ✅ User can run evolution
- ✅ User can see progress during execution
- ✅ User can view results in table
- ✅ User can click strategy to see equity chart
- ✅ User can export results

### Production Ready
- MVP +
- ✅ Config save/load
- ✅ Error messages are clear and actionable
- ✅ Large datasets don't freeze UI
- ✅ All inputs validated with inline feedback
- ✅ Keyboard shortcuts work
- ✅ App is responsive and polished

---

## Next Steps

1. **Read the detailed documents**:
   - `00-ui-strategy-generation.md` - Full specifications
   - `01-csv-data-connector.md` - Backend module 1
   - `02-ast-pretty-printer.md` - Backend module 2

2. **Implement Phase 0** (backend prerequisites)

3. **Begin Phase 1** (foundation)

4. **Iterate through phases** 2-5

5. **Test and refine**

---

## Support

For detailed implementation guidance, refer to:
- **Full UI Spec**: `00-ui-strategy-generation.md`
- **CSV Connector**: `01-csv-data-connector.md`
- **AST Formatter**: `02-ast-pretty-printer.md`

For egui examples:
- [egui Demo](https://www.egui.rs/)
- [eframe Examples](https://github.com/emilk/egui/tree/master/examples)

---

**Status**: Ready to implement
**Estimated Total Time**: 5-7 weeks (including backend prerequisites)
**Last Updated**: 2025-11-15
