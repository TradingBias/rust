use eframe::NativeOptions;
use tradebias::ui::TradeBiasApp;

fn main() -> eframe::Result<()> {
    // Configure logging (optional)
    env_logger::init();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_title("TradeBias - Strategy Generator"),
        ..Default::default()
    };

    eframe::run_native(
        "TradeBias",
        native_options,
        Box::new(|cc| Ok(Box::new(TradeBiasApp::new(cc)))),
    )
}
