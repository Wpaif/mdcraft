#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Oculta o console no Windows em release

use eframe::egui;

mod app;
mod model;
mod parse;
mod theme;
mod units;

use crate::app::MdcraftApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 750.0])
            .with_min_inner_size([600.0, 500.0]),

        ..Default::default()
    };

    eframe::run_native(
        "Mdcraft Calculator Pro",
        options,
        Box::new(|_cc| Ok(Box::<MdcraftApp>::default())),
    )
}
