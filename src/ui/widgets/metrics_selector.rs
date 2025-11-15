use crate::ui::state::{AppState, MetricInfo};
use crate::engines::generation::pareto::OptimizationDirection;

/// Metrics selector widget for multi-objective optimization
pub struct MetricsSelector;

impl MetricsSelector {
    pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Optimization Metrics");
        ui.add_space(5.0);

        ui.label("Select metrics to optimize (Pareto Frontier):");
        ui.add_space(5.0);

        // Show available metrics with checkboxes
        for metric in &state.available_metrics.clone() {
            ui.horizontal(|ui| {
                let is_selected = state.selected_metrics.contains_key(&metric.name);

                // Checkbox for selection
                let mut selected = is_selected;
                if ui.checkbox(&mut selected, &metric.display_name).changed() {
                    if selected {
                        // Add metric with default direction
                        state.selected_metrics.insert(
                            metric.name.clone(),
                            metric.default_direction,
                        );
                    } else {
                        // Remove metric
                        state.selected_metrics.remove(&metric.name);
                    }
                }

                // If selected, show direction toggle
                if is_selected {
                    let current_direction = state.selected_metrics.get(&metric.name).copied()
                        .unwrap_or(metric.default_direction);

                    ui.label("|");

                    let mut is_maximize = matches!(current_direction, OptimizationDirection::Maximize);
                    if ui.radio(is_maximize, "Maximize").clicked() {
                        state.selected_metrics.insert(
                            metric.name.clone(),
                            OptimizationDirection::Maximize,
                        );
                    }

                    ui.label("/");

                    if ui.radio(!is_maximize, "Minimize").clicked() {
                        state.selected_metrics.insert(
                            metric.name.clone(),
                            OptimizationDirection::Minimize,
                        );
                    }
                }

                // Tooltip with description
                ui.label("ℹ").on_hover_text(&metric.description);
            });
        }

        ui.add_space(10.0);

        // Show summary of selected metrics
        if !state.selected_metrics.is_empty() {
            ui.separator();
            ui.label(format!(
                "Selected: {} metric{}",
                state.selected_metrics.len(),
                if state.selected_metrics.len() == 1 { "" } else { "s" }
            ));

            ui.add_space(5.0);

            // Display selected metrics
            for (metric_name, direction) in &state.selected_metrics {
                if let Some(metric_info) = state.available_metrics.iter().find(|m| &m.name == metric_name) {
                    let dir_str = match direction {
                        OptimizationDirection::Maximize => "↑ Max",
                        OptimizationDirection::Minimize => "↓ Min",
                    };
                    ui.label(format!("  • {} ({})", metric_info.display_name, dir_str));
                }
            }
        } else {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ No metrics selected! Please select at least one metric."
            );
        }
    }
}
