use crate::ui::state::AppState;
use crate::ui::widgets::StrategyTable;

pub struct MainPanel;

impl MainPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Strategy Results");
        ui.separator();

        // Progress bar (shown when running)
        if state.is_running {
            ui.label(format!(
                "Generation {}/{}",
                state.current_generation, state.num_generations
            ));
            ui.add(
                egui::ProgressBar::new(state.progress_percentage)
                    .show_percentage()
                    .animate(true),
            );
        }

        // Status message
        ui.horizontal(|ui| {
            let color = if state.status_message.contains("Error") || state.status_message.contains("error") {
                egui::Color32::RED
            } else if state.is_running {
                egui::Color32::YELLOW
            } else {
                egui::Color32::GREEN
            };
            ui.colored_label(color, &state.status_message);
        });

        ui.separator();

        // Results table
        if state.hall_of_fame.is_empty() {
            ui.centered_and_justified(|ui| {
                if state.is_running {
                    ui.label("Evolution in progress...");
                } else {
                    ui.label("No strategies yet. Configure settings and click 'Run Evolution' to start.");
                }
            });
        } else {
            ui.label(format!("Hall of Fame: {} strategies", state.hall_of_fame.len()));
            ui.separator();
            StrategyTable::show(ui, state);
        }
    }
}
