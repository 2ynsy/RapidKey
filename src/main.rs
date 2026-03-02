#![windows_subsystem = "windows"]

mod theme;
mod models;
mod utils;
mod engine;
mod ui;

use crossbeam_channel::unbounded;
use eframe::egui;
use models::AppState;
use ui::FastPulseKeyUI;
use utils::{setup_fonts, setup_theme};

fn main() -> eframe::Result {
    let app_state = AppState::default();
    let (tx_start, rx_start) = unbounded::<()>();
    let (tx_stop, rx_stop) = unbounded::<()>();

    // Spawn Engine Threads
    engine::spawn_fire_engine(app_state.clone(), rx_start, rx_stop);
    engine::spawn_event_listener(app_state.clone(), tx_start.clone(), tx_stop.clone());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 580.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "FastPulseKey ⚡",
        options,
        Box::new(|cc| {
            setup_theme(&cc.egui_ctx);
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(FastPulseKeyUI::new(app_state, tx_start, tx_stop)))
        }),
    )
}
