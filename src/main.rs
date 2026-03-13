#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Oculta o console no Windows em release

use eframe::egui;
use resvg::{tiny_skia, usvg};

mod app;
mod model;
mod parse;
mod theme;
mod units;

use crate::app::MdcraftApp;

const APP_TITLE: &str = "Mdcraft";
const APP_ID: &str = "mdcraft";

fn load_app_icon() -> Option<egui::IconData> {
    let svg_data = include_bytes!("../assets/icon.svg");

    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &opts).ok()?;
    let size = tree.size().to_int_size();

    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;
    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap_mut);

    Some(egui::IconData {
        rgba: pixmap.data().to_vec(),
        width: size.width(),
        height: size.height(),
    })
}

#[cfg(target_os = "linux")]
fn ensure_linux_desktop_integration() {
    use std::fs;
    use std::path::PathBuf;

    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));

    let Some(data_home) = data_home else {
        return;
    };

    let applications_dir = data_home.join("applications");
    let icons_dir = data_home.join("icons/hicolor/scalable/apps");

    let _ = fs::create_dir_all(&applications_dir);
    let _ = fs::create_dir_all(&icons_dir);

    let icon_path = icons_dir.join(format!("{APP_ID}.svg"));
    let _ = fs::write(&icon_path, include_bytes!("../assets/icon.svg"));

    let Ok(exec_path) = std::env::current_exe() else {
        return;
    };

    let desktop_path = applications_dir.join(format!("{APP_ID}.desktop"));
    let desktop_entry = format!(
        "[Desktop Entry]\nType=Application\nName={APP_TITLE}\nExec={}\nIcon={APP_ID}\nTerminal=false\nCategories=Utility;\nStartupWMClass={APP_ID}\nStartupNotify=true\n"
    , exec_path.display());

    let _ = fs::write(desktop_path, desktop_entry);
}

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "linux")]
    ensure_linux_desktop_integration();

    let viewport = {
        let viewport = egui::ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_app_id(APP_ID)
            .with_inner_size([1000.0, 750.0])
            .with_min_inner_size([600.0, 500.0]);

        if let Some(icon) = load_app_icon() {
            viewport.with_icon(icon)
        } else {
            viewport
        }
    };

    let options = eframe::NativeOptions {
        viewport,

        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(MdcraftApp::from_creation_context(cc)))),
    )
}
