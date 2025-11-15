use crate::ui::state::{AppState, StrategyDisplay};

pub struct StrategyTable;

impl StrategyTable {
    pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
        if state.hall_of_fame.is_empty() {
            return;
        }

        // Clone sort settings to avoid borrow issues
        let sort_column = state.sort_column.clone();
        let sort_ascending = state.sort_ascending;

        // Sort strategies based on current sort settings
        let mut sorted_strategies: Vec<_> = state.hall_of_fame.iter().enumerate().collect();

        sorted_strategies.sort_by(|(_, a), (_, b)| {
            let ordering = match sort_column.as_str() {
                "rank" => a.rank.cmp(&b.rank),
                "fitness" => a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal),
                "return" => a.return_pct.partial_cmp(&b.return_pct).unwrap_or(std::cmp::Ordering::Equal),
                "trades" => a.total_trades.cmp(&b.total_trades),
                "winrate" => a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal),
                "drawdown" => a.max_drawdown.partial_cmp(&b.max_drawdown).unwrap_or(std::cmp::Ordering::Equal),
                "sharpe" => a.sharpe_ratio.partial_cmp(&b.sharpe_ratio).unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            };

            if sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        // Clone data needed for headers to avoid borrow conflicts
        let mut selected_idx = state.selected_strategy_idx;
        let mut sort_column = state.sort_column.clone();
        let mut sort_ascending = state.sort_ascending;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Header row with clickable columns for sorting
            ui.horizontal(|ui| {
                if Self::sortable_header_button(ui, "Rank", "rank", &sort_column, sort_ascending).clicked() {
                    if sort_column == "rank" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "rank".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Fitness", "fitness", &sort_column, sort_ascending).clicked() {
                    if sort_column == "fitness" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "fitness".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Return %", "return", &sort_column, sort_ascending).clicked() {
                    if sort_column == "return" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "return".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Trades", "trades", &sort_column, sort_ascending).clicked() {
                    if sort_column == "trades" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "trades".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Win Rate %", "winrate", &sort_column, sort_ascending).clicked() {
                    if sort_column == "winrate" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "winrate".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Max DD %", "drawdown", &sort_column, sort_ascending).clicked() {
                    if sort_column == "drawdown" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "drawdown".to_string();
                        sort_ascending = false;
                    }
                }
                if Self::sortable_header_button(ui, "Sharpe", "sharpe", &sort_column, sort_ascending).clicked() {
                    if sort_column == "sharpe" {
                        sort_ascending = !sort_ascending;
                    } else {
                        sort_column = "sharpe".to_string();
                        sort_ascending = false;
                    }
                }
                ui.label("Formula");
            });

            ui.separator();

            egui::Grid::new("strategy_table")
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {

                    // Data rows
                    for (original_idx, strategy) in sorted_strategies {
                        let is_selected = selected_idx == Some(original_idx);

                        if ui.selectable_label(is_selected, format!("{}", strategy.rank))
                            .clicked()
                        {
                            selected_idx = Some(original_idx);
                        }

                        ui.label(format!("{:.4}", strategy.fitness));
                        ui.label(format!("{:.2}", strategy.return_pct));
                        ui.label(format!("{}", strategy.total_trades));
                        ui.label(format!("{:.2}", strategy.win_rate));
                        ui.label(format!("{:.2}", strategy.max_drawdown));
                        ui.label(format!("{:.2}", strategy.sharpe_ratio));

                        // Formula with tooltip showing full version
                        let formula_short = strategy.formula.clone();
                        ui.label(&formula_short).on_hover_text(&strategy.formula_full);

                        ui.end_row();
                    }
                });
        });

        // Update the state with new selections
        state.selected_strategy_idx = selected_idx;
        state.sort_column = sort_column;
        state.sort_ascending = sort_ascending;
    }

    fn sortable_header_button(ui: &mut egui::Ui, label: &str, column: &str, current_sort: &str, sort_ascending: bool) -> egui::Response {
        let is_current = current_sort == column;
        let arrow = if is_current {
            if sort_ascending { " ▲" } else { " ▼" }
        } else {
            ""
        };

        ui.button(format!("{}{}", label, arrow))
    }
}
