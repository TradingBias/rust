use crate::config::backtesting::ValidationMethod;
use crate::config::trade_management::{StopLossConfig, TakeProfitConfig, PositionSizing};
use crate::ui::state::AppState;
use crate::ui::widgets::{DataSelector, IndicatorSelector, MetricsSelector};

pub struct LeftPanel;

impl LeftPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Configuration");
        ui.separator();

        // 1. Data Selection Section
        ui.collapsing("Data Selection", |ui| {
            DataSelector::show(ui, state);
        });

        ui.separator();

        // 2. Indicator Selection Section
        ui.collapsing("Indicators", |ui| {
            IndicatorSelector::show(ui, state);
        });

        ui.separator();

        // 3. Optimization Metrics Section
        ui.collapsing("Optimization Metrics", |ui| {
            MetricsSelector::show(ui, state);
        });

        ui.separator();

        // 4. Trade Management Section
        ui.collapsing("Trade Management", |ui| {
            Self::show_trade_management(ui, state);
        });

        ui.separator();

        // 4. Evolution & Backtesting Section
        ui.collapsing("Evolution & Backtesting", |ui| {
            Self::show_evolution_config(ui, state);
            ui.separator();
            Self::show_backtesting_config(ui, state);
        });

        ui.separator();

        // 5. Control Buttons
        Self::show_control_buttons(ui, state);
    }

    fn show_trade_management(ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            ui.label("Initial Capital:");
            ui.add(egui::DragValue::new(&mut state.initial_capital)
                .prefix("$")
                .range(100.0..=1_000_000.0));
        });

        ui.horizontal(|ui| {
            ui.label("Commission:");
            ui.add(egui::DragValue::new(&mut state.commission)
                .suffix("%")
                .speed(0.001)
                .range(0.0..=1.0)
                .fixed_decimals(3));
        });

        ui.horizontal(|ui| {
            ui.label("Slippage:");
            ui.add(egui::DragValue::new(&mut state.slippage)
                .suffix("%")
                .speed(0.001)
                .range(0.0..=1.0)
                .fixed_decimals(3));
        });

        ui.horizontal(|ui| {
            ui.label("Position Sizing:");
            egui::ComboBox::from_id_salt("position_sizing")
                .selected_text(format!("{:?}", state.position_sizing))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.position_sizing, PositionSizing::Fixed { size: 100.0 }, "Fixed");
                    ui.selectable_value(&mut state.position_sizing, PositionSizing::Percent { percent: 1.0 }, "Percent");
                    ui.selectable_value(&mut state.position_sizing, PositionSizing::Kelly { fraction: 0.25 }, "Kelly");
                });
        });

        // Show size input based on selected method
        match &mut state.position_sizing {
            PositionSizing::Fixed { size } => {
                ui.horizontal(|ui| {
                    ui.label("  Size:");
                    ui.add(egui::DragValue::new(size).prefix("$").range(1.0..=100_000.0));
                });
            }
            PositionSizing::Percent { percent } => {
                ui.horizontal(|ui| {
                    ui.label("  Percent:");
                    ui.add(egui::DragValue::new(percent).suffix("%").range(0.1..=100.0));
                });
            }
            PositionSizing::Kelly { fraction } => {
                ui.horizontal(|ui| {
                    ui.label("  Fraction:");
                    ui.add(egui::DragValue::new(fraction).range(0.1..=1.0).speed(0.05));
                });
            }
        }

        ui.horizontal(|ui| {
            ui.label("Max Positions:");
            ui.add(egui::DragValue::new(&mut state.max_positions).range(1..=20));
        });

        ui.label("Stop Loss:");
        egui::ComboBox::from_id_salt("stop_loss")
            .selected_text(format!("{:?}", state.stop_loss))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.stop_loss, StopLossConfig::None, "None");
                ui.selectable_value(&mut state.stop_loss, StopLossConfig::FixedPercent { percent: 2.0 }, "Fixed Percent");
                ui.selectable_value(&mut state.stop_loss, StopLossConfig::ATR { multiplier: 2.0, period: 14 }, "ATR");
            });

        // Show SL parameters based on selected method
        match &mut state.stop_loss {
            StopLossConfig::None => {}
            StopLossConfig::FixedPercent { percent } => {
                ui.horizontal(|ui| {
                    ui.label("  Percent:");
                    ui.add(egui::DragValue::new(percent).suffix("%").range(0.1..=50.0));
                });
            }
            StopLossConfig::ATR { multiplier, period } => {
                ui.horizontal(|ui| {
                    ui.label("  Multiplier:");
                    ui.add(egui::DragValue::new(multiplier).range(0.5..=10.0).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("  Period:");
                    ui.add(egui::DragValue::new(period).range(5..=50));
                });
            }
        }

        ui.label("Take Profit:");
        egui::ComboBox::from_id_salt("take_profit")
            .selected_text(format!("{:?}", state.take_profit))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.take_profit, TakeProfitConfig::None, "None");
                ui.selectable_value(&mut state.take_profit, TakeProfitConfig::FixedPercent { percent: 5.0 }, "Fixed Percent");
                ui.selectable_value(&mut state.take_profit, TakeProfitConfig::RiskReward { ratio: 2.0 }, "Risk/Reward");
            });

        // Show TP parameters based on selected method
        match &mut state.take_profit {
            TakeProfitConfig::None => {}
            TakeProfitConfig::FixedPercent { percent } => {
                ui.horizontal(|ui| {
                    ui.label("  Percent:");
                    ui.add(egui::DragValue::new(percent).suffix("%").range(0.1..=100.0));
                });
            }
            TakeProfitConfig::RiskReward { ratio } => {
                ui.horizontal(|ui| {
                    ui.label("  Ratio:");
                    ui.add(egui::DragValue::new(ratio).range(0.5..=10.0).speed(0.1));
                });
            }
        }
    }

    fn show_evolution_config(ui: &mut egui::Ui, state: &mut AppState) {
        ui.label("Evolution Parameters:");

        ui.horizontal(|ui| {
            ui.label("Population:");
            ui.add(egui::DragValue::new(&mut state.population_size).range(50..=5000));
        });

        ui.horizontal(|ui| {
            ui.label("Generations:");
            ui.add(egui::DragValue::new(&mut state.num_generations).range(10..=1000));
        });

        ui.horizontal(|ui| {
            ui.label("Mutation Rate:");
            ui.add(egui::Slider::new(&mut state.mutation_rate, 0.0..=1.0).step_by(0.01));
        });

        ui.horizontal(|ui| {
            ui.label("Crossover Rate:");
            ui.add(egui::Slider::new(&mut state.crossover_rate, 0.0..=1.0).step_by(0.01));
        });

        ui.horizontal(|ui| {
            ui.label("Elitism Count:");
            ui.add(egui::DragValue::new(&mut state.elitism_count).range(1..=50));
        });

        ui.horizontal(|ui| {
            ui.label("Max Tree Depth:");
            ui.add(egui::DragValue::new(&mut state.max_tree_depth).range(3..=20));
        });

        ui.horizontal(|ui| {
            ui.label("Tournament Size:");
            ui.add(egui::DragValue::new(&mut state.tournament_size).range(2..=20));
        });
    }

    fn show_backtesting_config(ui: &mut egui::Ui, state: &mut AppState) {
        ui.label("Backtesting Parameters:");

        ui.horizontal(|ui| {
            ui.label("Validation Method:");
            egui::ComboBox::from_id_salt("validation_method")
                .selected_text(format!("{:?}", state.validation_method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.validation_method, ValidationMethod::Simple, "Simple");
                    ui.selectable_value(&mut state.validation_method, ValidationMethod::WalkForwardAnchored, "Walk Forward Anchored");
                    ui.selectable_value(&mut state.validation_method, ValidationMethod::WalkForwardRolling, "Walk Forward Rolling");
                    ui.selectable_value(&mut state.validation_method, ValidationMethod::KFold, "K-Fold");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Train/Test Split:");
            ui.add(egui::Slider::new(&mut state.train_test_split, 0.5..=0.9).step_by(0.05));
        });

        if state.validation_method == ValidationMethod::KFold {
            ui.horizontal(|ui| {
                ui.label("Num Folds:");
                ui.add(egui::DragValue::new(&mut state.num_folds).range(2..=10));
            });
        }
    }

    fn show_control_buttons(ui: &mut egui::Ui, state: &mut AppState) {
        ui.vertical_centered(|ui| {
            // Validate before allowing run
            let can_run = Self::validate_config(state).is_ok() && !state.is_running;
            let validation_error = Self::validate_config(state).err();

            let run_button = ui.add_enabled(can_run, egui::Button::new("▶ Run Evolution"));
            if run_button.clicked() {
                state.status_message = "Starting evolution...".to_string();
                // The actual start will be handled in app.rs
            }

            if let Some(error) = validation_error {
                ui.colored_label(egui::Color32::RED, error);
            }

            let stop_button = ui.add_enabled(state.is_running, egui::Button::new("⏹ Stop"));
            if stop_button.clicked() {
                state.status_message = "Stopping...".to_string();
                // The actual stop will be handled in app.rs
            }
        });
    }

    fn validate_config(state: &AppState) -> Result<(), String> {
        if state.loaded_data.is_none() {
            return Err("No data loaded".to_string());
        }

        if state.selected_indicators.is_empty() {
            return Err("No indicators selected".to_string());
        }

        if state.initial_capital <= 0.0 {
            return Err("Invalid initial capital".to_string());
        }

        Ok(())
    }
}
