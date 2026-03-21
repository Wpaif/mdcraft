//! Top-level application logic and state.
//!
//! The previous `src/app.rs` file has been broken into smaller submodules
//! following a directory-based layout.  Each feature of the UI has its own
//! file:
//!
//! * `theme_state` – theme enum and toggle button logic.
//! * `styles` – custom egui styling and emoji font support.
//! * `price` – price status indicator helpers.
//! * `sidebar` – docked collapsible left menu.
//! * `ui` – the implementation of `eframe::App` and the main view.
//!
//! Visibility conventions:
//! - `pub`: only stable app-facing types/re-exports intended for cross-module use.
//! - `pub(crate)`: shared internals used across `src/app` submodules.
//! - private: implementation details that should not leak outside the module.

use dark_light::Mode;
use eframe::egui;

// `Item` is needed for the application state. Parser functions and formatter are
// now imported in `ui.rs` where the UI logic lives.

// re-export the Theme type so callers can refer to `app::Theme`.
pub use state::MdcraftApp;
pub use theme_state::Theme;

// re-export craft inference helpers used across the app.
pub(crate) use craft_inference::{build_craft_recipe_name_index, infer_craft_name_from_items};
pub use saved_craft::{SavedCraft, SavedItemPrice};
pub(crate) use saved_prices::{apply_saved_item_prices, capture_saved_item_prices};

/// Title-cases every word in `raw_name` (trims surrounding whitespace, collapses
/// inner runs). Shared by sidebar and item-grid display logic.
pub(crate) fn capitalize_display_name(raw_name: &str) -> String {
    raw_name
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            let first = chars
                .next()
                .expect("split_whitespace yields non-empty words");
            format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Dimmed placeholder text used inside egui hint texts.
pub(crate) fn placeholder(ui: &egui::Ui, text: &str) -> egui::RichText {
    egui::RichText::new(text).color(ui.visuals().text_color().gamma_multiply(0.7))
}

#[path = "domain/craft_inference.rs"]
mod craft_inference;
#[path = "domain/npc_price_rules.rs"]
pub(crate) mod npc_price_rules;
#[path = "domain/sell_price.rs"]
pub(crate) mod sell_price;
#[path = "presentation/price.rs"]
mod price;
#[path = "core/saved_craft.rs"]
mod saved_craft;
#[path = "core/saved_prices.rs"]
mod saved_prices;
#[path = "core/settings.rs"]
mod settings;
mod sidebar;
#[path = "core/sqlite.rs"]
mod sqlite;
#[path = "core/state.rs"]
mod state;
#[path = "presentation/styles.rs"]
mod styles;
#[path = "presentation/theme_state.rs"]
mod theme_state;
#[path = "presentation/theme_toggle.rs"]
mod theme_toggle;
#[path = "presentation/ui.rs"]
mod ui;
mod ui_sections;

#[cfg(test)]
pub(crate) use settings::APP_SETTINGS_KEY;
#[cfg(test)]
use settings::AppSettings;

pub(super) fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        Ok(Mode::Dark) => Theme::Dark,
        Ok(Mode::Light) | Ok(Mode::Unspecified) | Err(_) => Theme::Light,
    }
}

impl MdcraftApp {
    #[cfg(test)]
    pub(crate) fn rebuild_craft_recipe_name_index(&mut self) {
        self.craft_recipe_name_by_signature =
            build_craft_recipe_name_index(&self.craft_recipes_cache);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::{CreationContext, Storage, egui};

    use super::{
        APP_SETTINGS_KEY, AppSettings, MdcraftApp, SavedCraft, Theme, detect_system_theme,
    };
    use crate::data::wiki_scraper::{ScrapedItem, WikiSource};

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_string(), value);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn default_app_starts_with_expected_flags() {
        let app = MdcraftApp::default();
        assert!(app.follow_system_theme);
        assert!(app.sidebar_open);
        assert!(!app.fonts_loaded);
        assert!(app.items.is_empty());
    }

    #[test]
    fn save_app_settings_writes_json_payload() {
        let mut app = MdcraftApp::default();
        app.theme = Theme::Dark;
        app.follow_system_theme = false;
        app.saved_crafts.push(SavedCraft {
            name: "Receita A".to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "10k".to_string(),
            sell_price_is_per_item: false,
            item_prices: vec![],
        });

        let mut storage = MemoryStorage::default();
        app.save_app_settings(&mut storage);

        let raw = storage
            .get_string(APP_SETTINGS_KEY)
            .expect("settings JSON should be stored");
        assert!(raw.contains("Receita A"));

        storage.flush();

        storage.flush();
    }

    #[test]
    fn from_creation_context_restores_saved_settings() {
        let settings = AppSettings {
            theme: Some(Theme::Dark),
            follow_system_theme: Some(false),
            saved_crafts: vec![SavedCraft {
                name: "Restaurada".to_string(),
                recipe_text: "2 Screw".to_string(),
                sell_price_input: "4k".to_string(),
                sell_price_is_per_item: false,
                item_prices: vec![],
            }],
            wiki_cached_items: vec![],
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_last_sync_unix_seconds: None,
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.theme, Theme::Dark);
        assert!(!app.follow_system_theme);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.saved_crafts[0].name, "Restaurada");
    }

    #[test]
    fn detect_system_theme_returns_valid_variant() {
        let theme = detect_system_theme();
        assert!(matches!(theme, Theme::Light | Theme::Dark));
    }

    #[test]
    fn from_creation_context_ignores_invalid_settings_json() {
        let mut storage = MemoryStorage::default();
        storage.set_string(APP_SETTINGS_KEY, "{invalid-json".to_string());

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert!(app.saved_crafts.is_empty());
    }

    #[test]
    fn from_creation_context_applies_partial_settings() {
        let settings = AppSettings {
            theme: Some(Theme::Dark),
            follow_system_theme: None,
            saved_crafts: vec![],
            wiki_cached_items: vec![],
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_last_sync_unix_seconds: None,
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.theme, Theme::Dark);
        assert!(app.follow_system_theme);
    }

    #[test]
    fn from_creation_context_restores_wiki_cache_and_keeps_resource_seed() {
        let settings = AppSettings {
            theme: None,
            follow_system_theme: None,
            saved_crafts: vec![],
            wiki_cached_items: vec![
                ScrapedItem {
                    name: "Ancient Wire".to_string(),
                    npc_price: Some("12k".to_string()),
                    sources: vec![WikiSource::Loot],
                },
                ScrapedItem {
                    name: "Gear Nose".to_string(),
                    npc_price: None,
                    sources: vec![WikiSource::Nightmare],
                },
            ],
            wiki_http_etag_cache: HashMap::from([(
                String::from("https://wiki"),
                String::from("etag1"),
            )]),
            wiki_http_last_modified_cache: HashMap::from([(
                String::from("https://wiki"),
                String::from("Wed, 21 Oct 2015 07:28:00 GMT"),
            )]),
            wiki_last_sync_unix_seconds: Some(1_700_000_000),
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.wiki_cached_items.len(), 2);
        assert!(app.resource_list.contains(&"tech data".to_string()));
        assert!(!app.resource_list.contains(&"ancient wire".to_string()));
        assert_eq!(
            app.wiki_http_etag_cache
                .get("https://wiki")
                .map(String::as_str),
            Some("etag1")
        );
        assert_eq!(
            app.wiki_http_last_modified_cache
                .get("https://wiki")
                .map(String::as_str),
            Some("Wed, 21 Oct 2015 07:28:00 GMT")
        );
        assert_eq!(app.wiki_last_sync_unix_seconds, Some(1_700_000_000));
    }
}
