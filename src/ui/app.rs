use super::panels::{LeftPanel, MainPanel, RightPanel};
use super::services::{ConfigBridge, EvolutionRunner};
use super::state::{AppState, IndicatorInfo, IndicatorCategory};
use crate::functions::registry::FunctionRegistry;

pub struct TradeBiasApp {
    state: AppState,
    left_panel: LeftPanel,
    main_panel: MainPanel,
    right_panel: RightPanel,
    evolution_runner: Option<EvolutionRunner>,
    run_requested: bool,
    stop_requested: bool,
}

impl Default for TradeBiasApp {
    fn default() -> Self {
        let mut state = AppState::new();

        // Initialize available indicators from function registry
        let registry = FunctionRegistry::new();
        state.available_indicators = Self::get_available_indicators(&registry);

        Self {
            state,
            left_panel: LeftPanel::new(),
            main_panel: MainPanel::new(),
            right_panel: RightPanel::new(),
            evolution_runner: None,
            run_requested: false,
            stop_requested: false,
        }
    }
}

impl TradeBiasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        Self::default()
    }

    fn get_available_indicators(registry: &FunctionRegistry) -> Vec<IndicatorInfo> {
        registry
            .get_indicators()
            .iter()
            .map(|indicator| {
                let alias_str = indicator.alias();
                // Categorize indicators based on alias
                let category = if ["SMA", "EMA", "WMA", "DEMA", "TEMA", "KAMA", "HMA", "MACD", "SAR", "Bears", "Bulls", "TriX", "BB", "Envelopes"]
                    .contains(&alias_str)
                {
                    IndicatorCategory::Trend
                } else if ["RSI", "Stochastic", "CCI", "MFI", "ROC", "TSI", "WilliamsR", "Momentum", "AC", "AO", "RVI", "DeMarker"]
                    .contains(&alias_str)
                {
                    IndicatorCategory::Momentum
                } else if ["ATR", "ADX", "StdDev"].contains(&alias_str) {
                    IndicatorCategory::Volatility
                } else {
                    IndicatorCategory::Volume
                };

                IndicatorInfo {
                    name: indicator.ui_name().to_string(),
                    alias: indicator.alias().to_string(),
                    category,
                    description: format!("{} indicator", indicator.ui_name()),
                }
            })
            .collect()
    }

    fn handle_run_button(&mut self) {
        if self.run_requested && !self.state.is_running {
            self.run_requested = false;

            // Get configs
            let evolution_config = ConfigBridge::to_evolution_config(&self.state);
            let backtesting_config = ConfigBridge::to_backtesting_config(&self.state);
            let trade_management_config = ConfigBridge::to_trade_management_config(&self.state);
            let objective_configs = ConfigBridge::to_objective_configs(&self.state);

            // Get data and indicators
            if let Some(data) = self.state.loaded_data.clone() {
                let selected_indicators: Vec<String> =
                    self.state.selected_indicators.iter().cloned().collect();

                // Start evolution
                self.evolution_runner = Some(EvolutionRunner::start(
                    data,
                    evolution_config,
                    backtesting_config,
                    trade_management_config,
                    selected_indicators,
                    objective_configs,
                ));

                self.state.is_running = true;
                self.state.current_generation = 0;
                self.state.progress_percentage = 0.0;
                self.state.status_message = "Evolution started".to_string();
            }
        }
    }

    fn handle_stop_button(&mut self) {
        if self.stop_requested && self.state.is_running {
            self.stop_requested = false;

            if let Some(ref mut runner) = self.evolution_runner {
                runner.cancel();
            }

            self.state.is_running = false;
            self.state.status_message = "Evolution stopped".to_string();
        }
    }

    fn poll_evolution_progress(&mut self) {
        if let Some(ref mut runner) = self.evolution_runner {
            // Poll for progress updates
            while let Some(update) = runner.poll_progress() {
                self.state.current_generation = update.generation;
                self.state.progress_percentage =
                    update.generation as f32 / update.total_generations as f32;
                self.state.status_message = update.status;
            }

            // Check for completion
            if let Some(result) = runner.try_get_results() {
                match result {
                    Ok(strategies) => {
                        self.state.hall_of_fame = strategies;
                        self.state.status_message =
                            format!("Evolution complete! {} strategies found", self.state.hall_of_fame.len());
                    }
                    Err(e) => {
                        self.state.status_message = format!("Evolution failed: {}", e);
                    }
                }

                self.state.is_running = false;
                self.state.progress_percentage = 1.0;
                self.evolution_runner = None;
            }
        }
    }
}

impl eframe::App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle evolution state
        self.handle_run_button();
        self.handle_stop_button();
        self.poll_evolution_progress();

        // Request repaint if evolution is running
        if self.state.is_running {
            ctx.request_repaint();
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("TradeBias - Strategy Generator");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("About").clicked() {
                        // Show about dialog
                    }
                });
            });
        });

        // Left Panel - Configuration
        egui::SidePanel::left("left_panel")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.left_panel.show(ui, &mut self.state);

                    // Check if Run button was clicked
                    if self.state.status_message == "Starting evolution..." {
                        self.run_requested = true;
                    }

                    // Check if Stop button was clicked
                    if self.state.status_message == "Stopping..." {
                        self.stop_requested = true;
                    }
                });
            });

        // Right Panel - Strategy Details
        egui::SidePanel::right("right_panel")
            .default_width(350.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.right_panel.show(ui, &self.state);
                });
            });

        // Central Panel - Results Table
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_panel.show(ui, &mut self.state);
        });
    }
}
