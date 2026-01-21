use eframe::egui;
use rust_gcpv_lynx_export::gui::GcpvApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GCPV Lynx Export",
        native_options,
        Box::new(|cc| Ok(Box::new(GcpvApp::new(cc)))),
    )
}
