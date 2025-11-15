use crate::data::{DataPreview, CsvConnector};
use crate::ui::state::AppState;
use polars::prelude::*;
use std::path::PathBuf;

pub struct DataSelector;

impl DataSelector {
    pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            if ui.button("Select CSV File...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CSV Files", &["csv"])
                    .pick_file()
                {
                    // Load the data
                    match Self::load_data(&path) {
                        Ok((df, preview)) => {
                            state.data_file_path = Some(path);
                            state.loaded_data = Some(df);
                            state.data_preview = Some(preview);
                            state.status_message = "Data loaded successfully".to_string();
                        }
                        Err(e) => {
                            state.status_message = format!("Error loading data: {}", e);
                        }
                    }
                }
            }
        });

        // Display current file info
        if let Some(path) = &state.data_file_path {
            ui.label(format!("File: {}", path.file_name().unwrap_or_default().to_string_lossy()));

            if let Some(preview) = &state.data_preview {
                ui.label(format!("Rows: {}", preview.metadata.num_rows));
                ui.label(format!("Columns: {}", preview.metadata.num_columns));

                // Show data preview in collapsible section
                ui.collapsing("Preview", |ui| {
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        egui::Grid::new("data_preview_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                // Header
                                for col_name in &preview.metadata.columns {
                                    ui.label(col_name);
                                }
                                ui.end_row();

                                // Rows
                                for row in &preview.first_rows {
                                    for cell in row {
                                        ui.label(cell);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            }
        } else {
            ui.label("No data loaded");
        }
    }

    fn load_data(path: &PathBuf) -> Result<(DataFrame, DataPreview), String> {
        // Load and validate CSV
        let (df, _column_map) = CsvConnector::load_and_validate(path, Some(100))
            .map_err(|e| e.to_string())?;

        // Create preview
        let preview = CsvConnector::create_preview(path, &df)
            .map_err(|e| e.to_string())?;

        // Normalize column names
        let normalized_df = CsvConnector::normalize_columns(df)
            .map_err(|e| e.to_string())?;

        Ok((normalized_df, preview))
    }
}
