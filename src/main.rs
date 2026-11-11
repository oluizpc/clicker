// Sem janela de console no build release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod icon;
mod input;
mod theme;
mod worker;

use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let config = config::Config::load();
    let shared = Arc::new(worker::Shared::new(config.active.clone()));

    worker::spawn_hotkey(shared.clone());
    worker::spawn_clicker(shared.clone());

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 600.0])
            .with_min_inner_size([680.0, 460.0])
            .with_title("DzkLP - Clicker")
            .with_icon(std::sync::Arc::new(icon::build())),
        ..Default::default()
    };

    let shared_ui = shared.clone();
    eframe::run_native(
        "DzkLP - Clicker",
        native_options,
        Box::new(move |cc| Box::new(app::App::new(cc, shared_ui, config))),
    )
}
