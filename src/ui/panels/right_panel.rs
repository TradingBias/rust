use crate::ui::state::AppState;

pub struct RightPanel;

impl RightPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("Strategy Details");

        ui.separator();

        // Show details only if strategy is selected
        if let Some(idx) = state.selected_strategy_idx {
            if let Some(strategy) = state.hall_of_fame.get(idx) {
                // Metrics Card
                ui.group(|ui| {
                    ui.heading("Metrics");
                    ui.horizontal(|ui| {
                        ui.label("Return:");
                        ui.label(format!("{:.2}%", strategy.return_pct));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sharpe:");
                        ui.label(format!("{:.2}", strategy.sharpe_ratio));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Max DD:");
                        ui.label(format!("{:.2}%", strategy.max_drawdown));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Win Rate:");
                        ui.label(format!("{:.2}%", strategy.win_rate));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Total Trades:");
                        ui.label(format!("{}", strategy.total_trades));
                    });
                });

                ui.separator();

                // Formula Display
                ui.collapsing("Formula", |ui| {
                    ui.label(&strategy.formula_full);
                });

                ui.separator();

                // Equity Chart Placeholder
                ui.group(|ui| {
                    ui.heading("Equity Curve");
                    ui.label("Chart will be displayed here using egui::plot");
                    ui.label(format!("Points: {}", strategy.equity_curve.len()));

                    // Simple plot placeholder (plot feature not available in this egui version)
                    if !strategy.equity_curve.is_empty() {
                        ui.label(format!("Initial: {:.2}", strategy.equity_curve.first().unwrap_or(&0.0)));
                        ui.label(format!("Final: {:.2}", strategy.equity_curve.last().unwrap_or(&0.0)));
                        // TODO: Add egui_plot or eframe plot feature for chart visualization
                    }
                });

                ui.separator();

                // Trade List Placeholder
                ui.collapsing("Trades", |ui| {
                    ui.label(format!("{} trades", strategy.trades.len()));
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a strategy from the results to view details");
            });
        }
    }
}
