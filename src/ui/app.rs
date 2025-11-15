use super::panels::{LeftPanel, MainPanel, RightPanel};
use super::state::AppState;

pub struct TradeBiasApp {
    state: AppState,
    left_panel: LeftPanel,
    main_panel: MainPanel,
    right_panel: RightPanel,
}

impl Default for TradeBiasApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            left_panel: LeftPanel::new(),
            main_panel: MainPanel::new(),
            right_panel: RightPanel::new(),
        }
    }
}

impl TradeBiasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Initialize available indicators list here (from function registry)
        Self::default()
    }
}

impl eframe::App for TradeBiasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar (optional)
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
