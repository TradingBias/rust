use crate::ui::state::{AppState, IndicatorInfo, IndicatorCategory};
use std::collections::HashSet;

pub struct IndicatorSelector;

impl IndicatorSelector {
    pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
        // Group indicators by category
        let mut trend_indicators = Vec::new();
        let mut momentum_indicators = Vec::new();
        let mut volatility_indicators = Vec::new();
        let mut volume_indicators = Vec::new();

        for indicator in &state.available_indicators {
            match indicator.category {
                IndicatorCategory::Trend => trend_indicators.push(indicator),
                IndicatorCategory::Momentum => momentum_indicators.push(indicator),
                IndicatorCategory::Volatility => volatility_indicators.push(indicator),
                IndicatorCategory::Volume => volume_indicators.push(indicator),
            }
        }

        ui.label(format!("Selected: {}", state.selected_indicators.len()));

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            // Trend indicators
            if !trend_indicators.is_empty() {
                ui.collapsing("Trend", |ui| {
                    for indicator in trend_indicators {
                        Self::show_indicator_checkbox(ui, indicator, &mut state.selected_indicators);
                    }
                });
            }

            // Momentum indicators
            if !momentum_indicators.is_empty() {
                ui.collapsing("Momentum", |ui| {
                    for indicator in momentum_indicators {
                        Self::show_indicator_checkbox(ui, indicator, &mut state.selected_indicators);
                    }
                });
            }

            // Volatility indicators
            if !volatility_indicators.is_empty() {
                ui.collapsing("Volatility", |ui| {
                    for indicator in volatility_indicators {
                        Self::show_indicator_checkbox(ui, indicator, &mut state.selected_indicators);
                    }
                });
            }

            // Volume indicators
            if !volume_indicators.is_empty() {
                ui.collapsing("Volume", |ui| {
                    for indicator in volume_indicators {
                        Self::show_indicator_checkbox(ui, indicator, &mut state.selected_indicators);
                    }
                });
            }
        });

        // Select all / Deselect all buttons
        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                for indicator in &state.available_indicators {
                    state.selected_indicators.insert(indicator.name.clone());
                }
            }
            if ui.button("Clear All").clicked() {
                state.selected_indicators.clear();
            }
        });
    }

    fn show_indicator_checkbox(
        ui: &mut egui::Ui,
        indicator: &IndicatorInfo,
        selected: &mut HashSet<String>,
    ) {
        let mut is_selected = selected.contains(&indicator.name);

        if ui.checkbox(&mut is_selected, &indicator.alias)
            .on_hover_text(&indicator.description)
            .changed()
        {
            if is_selected {
                selected.insert(indicator.name.clone());
            } else {
                selected.remove(&indicator.name);
            }
        }
    }
}
