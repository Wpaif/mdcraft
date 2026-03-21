#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Oculta o console no Windows em release

use eframe::egui;
use resvg::{tiny_skia, usvg};

#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

mod app;
mod data;
mod model;
mod parse;
mod theme;
mod units;

use crate::app::MdcraftApp;

const APP_TITLE: &str = "Mdcraft";
const APP_ID: &str = "mdcraft";

#[cfg(target_os = "linux")]
fn resolve_linux_data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

#[cfg(target_os = "linux")]
fn desktop_entry_for(exec_path: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={APP_TITLE}\nExec={}\nIcon={APP_ID}\nTerminal=false\nCategories=Utility;\nStartupWMClass={APP_ID}\nStartupNotify=true\n",
        exec_path.display()
    )
}

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

fn with_optional_icon(
    viewport: egui::ViewportBuilder,
    icon: Option<egui::IconData>,
) -> egui::ViewportBuilder {
    if let Some(icon) = icon {
        viewport.with_icon(icon)
    } else {
        viewport
    }
}

#[cfg(target_os = "linux")]
fn ensure_linux_desktop_integration() {
    ensure_linux_desktop_integration_with(resolve_linux_data_home(), std::env::current_exe().ok());
}

#[cfg(target_os = "linux")]
fn ensure_linux_desktop_integration_with(data_home: Option<PathBuf>, exec_path: Option<PathBuf>) {
    use std::fs;

    let Some(data_home) = data_home else {
        return;
    };

    let applications_dir = data_home.join("applications");
    let icons_dir = data_home.join("icons/hicolor/scalable/apps");

    let _ = fs::create_dir_all(&applications_dir);
    let _ = fs::create_dir_all(&icons_dir);

    let icon_path = icons_dir.join(format!("{APP_ID}.svg"));
    let _ = fs::write(&icon_path, include_bytes!("../assets/icon.svg"));

    let Some(exec_path) = exec_path else {
        return;
    };

    let desktop_path = applications_dir.join(format!("{APP_ID}.desktop"));
    let desktop_entry = desktop_entry_for(&exec_path);

    let _ = fs::write(desktop_path, desktop_entry);
}

fn run_app<R>(runner: R) -> eframe::Result<()>
where
    R: for<'a> FnOnce(&str, eframe::NativeOptions, eframe::AppCreator<'a>) -> eframe::Result<()>,
{
    #[cfg(target_os = "linux")]
    ensure_linux_desktop_integration();

    let options = build_native_options();
    run_with_runner(options, runner)
}

fn build_viewport() -> egui::ViewportBuilder {
    let viewport = egui::ViewportBuilder::default()
        .with_title(APP_TITLE)
        .with_app_id(APP_ID)
        .with_inner_size([1000.0, 750.0])
        .with_min_inner_size([600.0, 500.0]);

    with_optional_icon(viewport, load_app_icon())
}

fn build_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: build_viewport(),
        ..Default::default()
    }
}

fn create_app_creator<'app>() -> eframe::AppCreator<'app> {
    Box::new(|cc| Ok(Box::new(MdcraftApp::from_creation_context(cc))))
}

fn run_with_runner<R>(options: eframe::NativeOptions, runner: R) -> eframe::Result<()>
where
    R: for<'a> FnOnce(&str, eframe::NativeOptions, eframe::AppCreator<'a>) -> eframe::Result<()>,
{
    runner(APP_TITLE, options, create_app_creator())
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    run_app(eframe::run_native)
}

#[cfg(test)]
#[path = "main/tests.rs"]
mod tests;
