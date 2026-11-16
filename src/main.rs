#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod bank;
mod scenes;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DM Sounds")
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([760.0, 540.0])
            .with_icon(std::sync::Arc::new(theme::icon())),
        ..Default::default()
    };
    eframe::run_native(
        "DM Sounds",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
