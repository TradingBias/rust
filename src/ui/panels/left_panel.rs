use crate::ui::state::AppState;

pub struct LeftPanel;

impl LeftPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Configuration");

        ui.separator();

        // Data Selection Section (placeholder)
        ui.collapsing("Data Selection", |ui| {
            ui.label("CSV file selector will go here");
            if let Some(path) = &state.data_file_path {
                ui.label(format!("Selected: {}", path.display()));
            } else {
                ui.label("No data loaded");
            }
        });

        ui.separator();

        // Indicator Selection Section (placeholder)
        ui.collapsing("Indicators", |ui| {
            ui.label("Indicator checkboxes will go here");
            ui.label(format!("Selected: {}", state.selected_indicators.len()));
        });

        ui.separator();

        // Trade Management Section (placeholder)
        ui.collapsing("Trade Management", |ui| {
            ui.label("Trade config inputs will go here");
            ui.horizontal(|ui| {
                ui.label("Initial Capital:");
                ui.add(egui::DragValue::new(&mut state.initial_capital).prefix("$"));
            });
        });

        ui.separator();

        // Evolution Configuration Section (placeholder)
        ui.collapsing("Evolution & Backtesting", |ui| {
            ui.label("Evolution parameters will go here");
            ui.horizontal(|ui| {
                ui.label("Population:");
                ui.add(egui::DragValue::new(&mut state.population_size).range(50..=5000));
            });
            ui.horizontal(|ui| {
                ui.label("Generations:");
                ui.add(egui::DragValue::new(&mut state.num_generations).range(10..=1000));
            });
        });

        ui.separator();

        // Control Buttons
        ui.vertical_centered(|ui| {
            let run_button = ui.add_enabled(!state.is_running, egui::Button::new("Run Evolution"));
            if run_button.clicked() {
                state.status_message = "Evolution would start here".to_string();
            }

            let stop_button = ui.add_enabled(state.is_running, egui::Button::new("Stop"));
            if stop_button.clicked() {
                state.status_message = "Stopped".to_string();
            }
        });
    }
}
