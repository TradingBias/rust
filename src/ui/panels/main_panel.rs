use crate::ui::state::AppState;

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
            ui.label(format!("Generation {}/{}", state.current_generation, state.num_generations));
            ui.add(egui::ProgressBar::new(state.progress_percentage).show_percentage());
        }

        ui.label(&state.status_message);

        ui.separator();

        // Results table (placeholder)
        if state.hall_of_fame.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No strategies yet. Click 'Run Evolution' to start.");
            });
        } else {
            ui.label(format!("{} strategies in Hall of Fame", state.hall_of_fame.len()));

            // Simple table placeholder
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("strategy_table")
                    .striped(true)
                    .show(ui, |ui| {
                        // Header
                        ui.label("Rank");
                        ui.label("Fitness");
                        ui.label("Return %");
                        ui.label("Trades");
                        ui.label("Formula");
                        ui.end_row();

                        // Rows (placeholder - will be implemented later)
                        for (idx, strategy) in state.hall_of_fame.iter().enumerate() {
                            if ui.selectable_label(
                                state.selected_strategy_idx == Some(idx),
                                format!("{}", strategy.rank)
                            ).clicked() {
                                state.selected_strategy_idx = Some(idx);
                            }
                            ui.label(format!("{:.4}", strategy.fitness));
                            ui.label(format!("{:.2}%", strategy.return_pct));
                            ui.label(format!("{}", strategy.total_trades));
                            ui.label(&strategy.formula);
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
