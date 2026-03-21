use eframe::egui;
#[cfg(target_os = "linux")]
use std::ffi::OsString;
#[cfg(target_os = "linux")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::{
    APP_ID, build_native_options, build_viewport, create_app_creator,
    ensure_linux_desktop_integration, load_app_icon, run_app, run_with_runner,
    with_optional_icon,
};
#[cfg(target_os = "linux")]
use super::{
    desktop_entry_for, ensure_linux_desktop_integration_with, resolve_linux_data_home,
};
use eframe::CreationContext;

#[cfg(target_os = "linux")]
fn restore_env_var(name: &str, old_value: Option<OsString>) {
    if let Some(value) = old_value {
        unsafe { std::env::set_var(name, value) };
    } else {
        unsafe { std::env::remove_var(name) };
    }
}

#[test]
fn load_app_icon_produces_rgba_data() {
    let icon = load_app_icon().expect("icon should load from embedded SVG");
    assert!(icon.width > 0);
    assert!(icon.height > 0);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
}

#[cfg(target_os = "linux")]
#[test]
fn ensure_linux_desktop_integration_writes_files_to_xdg_data_home() {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock should not be poisoned");

    let old_xdg = std::env::var_os("XDG_DATA_HOME");

    let temp_root = std::env::temp_dir().join(format!(
        "mdcraft-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic against epoch")
            .as_nanos()
    ));

    std::fs::create_dir_all(&temp_root).expect("temp data dir should be created");
    unsafe {
        std::env::set_var("XDG_DATA_HOME", &temp_root);
    }

    ensure_linux_desktop_integration();

    let icon_path: PathBuf = temp_root.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg"));
    let desktop_path: PathBuf = temp_root.join(format!("applications/{APP_ID}.desktop"));

    assert!(icon_path.exists());
    assert!(desktop_path.exists());

    let desktop_content =
        std::fs::read_to_string(&desktop_path).expect("desktop file should be readable");
    assert!(desktop_content.contains("[Desktop Entry]"));
    assert!(desktop_content.contains("Name=Mdcraft"));
    assert!(desktop_content.contains("Icon=mdcraft"));

    restore_env_var("XDG_DATA_HOME", old_xdg);

    let _ = std::fs::remove_dir_all(temp_root);
}

#[cfg(target_os = "linux")]
#[test]
fn ensure_linux_desktop_integration_with_returns_early_without_data_home() {
    let temp_root = std::env::temp_dir().join(format!(
        "mdcraft-test-no-data-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic against epoch")
            .as_nanos()
    ));

    ensure_linux_desktop_integration_with(None, Some(PathBuf::from("/tmp/mdcraft")));

    assert!(!temp_root.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn ensure_linux_desktop_integration_with_returns_early_without_exec_path() {
    let temp_root = std::env::temp_dir().join(format!(
        "mdcraft-test-no-exec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic against epoch")
            .as_nanos()
    ));

    std::fs::create_dir_all(&temp_root).expect("temp data dir should be created");
    ensure_linux_desktop_integration_with(Some(temp_root.clone()), None);

    let desktop_path: PathBuf = temp_root.join(format!("applications/{APP_ID}.desktop"));
    assert!(!desktop_path.exists());

    let _ = std::fs::remove_dir_all(temp_root);
}

#[cfg(target_os = "linux")]
#[test]
fn restore_env_var_handles_some_and_none_values() {
    let old_xdg = std::env::var_os("XDG_DATA_HOME");

    restore_env_var("XDG_DATA_HOME", Some(OsString::from("/tmp/mdcraft-restore")));
    assert_eq!(
        std::env::var_os("XDG_DATA_HOME"),
        Some(OsString::from("/tmp/mdcraft-restore"))
    );

    restore_env_var("XDG_DATA_HOME", None);
    assert!(std::env::var_os("XDG_DATA_HOME").is_none());

    restore_env_var("XDG_DATA_HOME", old_xdg);
}

#[test]
fn build_viewport_and_native_options_do_not_panic() {
    let _viewport = build_viewport();
    let _options = build_native_options();
}

#[test]
fn with_optional_icon_handles_none_and_some_icon() {
    let _no_icon = with_optional_icon(egui::ViewportBuilder::default(), None);

    let icon = load_app_icon().expect("icon should load");
    let _with_icon = with_optional_icon(egui::ViewportBuilder::default(), Some(icon));
}

#[test]
fn create_app_creator_builds_app_instance() {
    let creator = create_app_creator();
    let cc = CreationContext::_new_kittest(egui::Context::default());
    let app = creator(&cc).expect("app creator should return app");
    drop(app);
}

#[test]
fn run_with_runner_passes_title_and_creator() {
    let options = build_native_options();

    let result = run_with_runner(options, |title, _options, creator| {
        assert_eq!(title, "Mdcraft");
        let cc = CreationContext::_new_kittest(egui::Context::default());
        let app = creator(&cc).expect("runner should receive valid creator");
        drop(app);
        Ok(())
    });

    assert!(result.is_ok());
}

#[test]
fn run_app_invokes_runner_with_expected_title() {
    let result = run_app(|title, _options, creator| {
        assert_eq!(title, "Mdcraft");
        let cc = CreationContext::_new_kittest(egui::Context::default());
        let app = creator(&cc).expect("runner should receive valid creator");
        drop(app);
        Ok(())
    });

    assert!(result.is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn desktop_entry_for_contains_expected_fields() {
    let entry = desktop_entry_for(Path::new("/tmp/mdcraft"));
    assert!(entry.contains("[Desktop Entry]"));
    assert!(entry.contains("Name=Mdcraft"));
    assert!(entry.contains("Exec=/tmp/mdcraft"));
}

#[cfg(target_os = "linux")]
#[test]
fn resolve_linux_data_home_prefers_xdg_variable() {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock should not be poisoned");

    let old_xdg = std::env::var_os("XDG_DATA_HOME");
    let old_home = std::env::var_os("HOME");

    unsafe {
        std::env::set_var("XDG_DATA_HOME", "/tmp/mdcraft-xdg");
        std::env::set_var("HOME", "/tmp/mdcraft-home");
    }

    let resolved = resolve_linux_data_home().expect("data home should resolve");
    assert_eq!(resolved, PathBuf::from("/tmp/mdcraft-xdg"));

    restore_env_var("XDG_DATA_HOME", old_xdg);
    restore_env_var("HOME", old_home);
}

#[cfg(target_os = "linux")]
#[test]
fn resolve_linux_data_home_falls_back_to_home_when_xdg_missing() {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock should not be poisoned");

    let old_xdg = std::env::var_os("XDG_DATA_HOME");
    let old_home = std::env::var_os("HOME");

    unsafe {
        std::env::remove_var("XDG_DATA_HOME");
        std::env::set_var("HOME", "/tmp/mdcraft-home-only");
    }

    let resolved = resolve_linux_data_home().expect("home fallback should resolve");
    assert_eq!(resolved, PathBuf::from("/tmp/mdcraft-home-only/.local/share"));

    restore_env_var("XDG_DATA_HOME", old_xdg);
    restore_env_var("HOME", old_home);
}

#[cfg(target_os = "linux")]
#[test]
fn resolve_linux_data_home_returns_none_without_env_vars() {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock should not be poisoned");

    let old_xdg = std::env::var_os("XDG_DATA_HOME");
    let old_home = std::env::var_os("HOME");

    unsafe {
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("HOME");
    }

    assert!(resolve_linux_data_home().is_none());

    restore_env_var("XDG_DATA_HOME", old_xdg);
    restore_env_var("HOME", old_home);
}

