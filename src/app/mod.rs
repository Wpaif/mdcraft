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

use crate::model::Item;
use crate::data::wiki_scraper::{
    ScrapeRefreshData, ScrapedItem, embedded_resource_names, embedded_wiki_items,
};
use crate::parse::parse_price_flag;
use dark_light::Mode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Instant;

// `Item` is needed for the application state. Parser functions and formatter are
// now imported in `ui.rs` where the UI logic lives.

// re-export the Theme type so callers can refer to `app::Theme`.
pub use theme_state::Theme;

const APP_SETTINGS_KEY: &str = "mdcraft.app_settings";

mod price;
mod sidebar;
mod styles;
mod theme_state;
mod ui;
mod ui_sections;

pub(super) fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        Ok(Mode::Dark) => Theme::Dark,
        Ok(Mode::Light) | Ok(Mode::Unspecified) | Err(_) => Theme::Light,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedItemPrice {
    pub item_name: String,
    pub price_input: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedCraft {
    pub name: String,
    pub recipe_text: String,
    pub sell_price_input: String,
    #[serde(default)]
    pub item_prices: Vec<SavedItemPrice>,
}

const FIXED_NPC_PRICE_COMPRESSED_NIGHTMARE_GEMS: &str = "25k";
const FIXED_NPC_PRICE_NEUTRAL_ESSENCE: &str = "1k";

pub fn fixed_npc_price_input(item_name: &str) -> Option<&'static str> {
    let normalized = item_name.trim().to_lowercase();
    match normalized.as_str() {
        "compressed nightmare gems" => Some(FIXED_NPC_PRICE_COMPRESSED_NIGHTMARE_GEMS),
        "neutral essence" => Some(FIXED_NPC_PRICE_NEUTRAL_ESSENCE),
        _ => None,
    }
}

fn normalized_item_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub fn capture_saved_item_prices(items: &[Item]) -> Vec<SavedItemPrice> {
    let mut result: Vec<SavedItemPrice> = items
        .iter()
        .filter_map(|item| {
            let price_input = item.preco_input.trim();
            if price_input.is_empty() {
                return None;
            }

            Some(SavedItemPrice {
                item_name: item.nome.clone(),
                price_input: price_input.to_string(),
            })
        })
        .collect();

    result.sort_by(|a, b| a.item_name.cmp(&b.item_name));
    result
}

pub fn apply_saved_item_prices(items: &mut [Item], saved_prices: &[SavedItemPrice]) {
    let lookup: HashMap<String, &str> = saved_prices
        .iter()
        .map(|saved| (normalized_item_key(&saved.item_name), saved.price_input.as_str()))
        .collect();

    for item in items {
        let key = normalized_item_key(&item.nome);
        let Some(saved_input) = lookup.get(&key).copied() else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(saved_input) else {
            continue;
        };

        item.preco_input = saved_input.to_string();
        item.preco_unitario = parsed;
        item.valor_total = parsed * item.quantidade as f64;
    }
}

/// The application state that is passed to `eframe`.
///
/// In GKT4 terms, this is the *model* for the main window; the view logic lives
/// in `ui.rs` and helpers are in other submodules.
pub struct MdcraftApp {
    pub input_text: String,
    pub items: Vec<Item>,
    pub sell_price_input: String,
    pub resource_list: Vec<String>,
    pub fonts_loaded: bool,
    pub theme: Theme,
    pub follow_system_theme: bool,
    pub sidebar_open: bool,
    pub saved_crafts: Vec<SavedCraft>,
    pub pending_craft_name: String,
    pub awaiting_craft_name: bool,
    pub focus_craft_name_input: bool,
    pub pending_delete_index: Option<usize>,
    pub active_saved_craft_index: Option<usize>,
    pub awaiting_import_json: bool,
    pub import_json_input: String,
    pub import_feedback: Option<String>,
    pub awaiting_export_json: bool,
    pub export_json_output: String,
    pub export_feedback: Option<String>,
    pub wiki_sync_feedback: Option<String>,
    pub wiki_cached_items: Vec<ScrapedItem>,
    pub wiki_http_etag_cache: HashMap<String, String>,
    pub wiki_http_last_modified_cache: HashMap<String, String>,
    pub wiki_refresh_in_progress: bool,
    pub wiki_refresh_rx: Option<Receiver<Result<ScrapeRefreshData, String>>>,
    pub wiki_sync_success_anim_started_at: Option<Instant>,
    pub wiki_refresh_started_on_launch: bool,
    pub wiki_last_sync_unix_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    theme: Option<Theme>,
    follow_system_theme: Option<bool>,
    #[serde(default)]
    saved_crafts: Vec<SavedCraft>,
    #[serde(default)]
    wiki_cached_items: Vec<ScrapedItem>,
    #[serde(default)]
    wiki_http_etag_cache: HashMap<String, String>,
    #[serde(default)]
    wiki_http_last_modified_cache: HashMap<String, String>,
    wiki_last_sync_unix_seconds: Option<u64>,
}

impl MdcraftApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        if let Some(storage) = cc.storage
            && let Some(raw) = storage.get_string(APP_SETTINGS_KEY)
            && let Ok(settings) = serde_json::from_str::<AppSettings>(&raw)
        {
            if let Some(theme) = settings.theme {
                app.theme = theme;
            }
            if let Some(follow_system_theme) = settings.follow_system_theme {
                app.follow_system_theme = follow_system_theme;
            }
            app.saved_crafts = settings.saved_crafts;
            if !settings.wiki_cached_items.is_empty() {
                app.wiki_cached_items = settings.wiki_cached_items;
            }
            app.wiki_http_etag_cache = settings.wiki_http_etag_cache;
            app.wiki_http_last_modified_cache = settings.wiki_http_last_modified_cache;
            app.wiki_last_sync_unix_seconds = settings.wiki_last_sync_unix_seconds;
        }

        app
    }

    pub fn save_app_settings(&self, storage: &mut dyn eframe::Storage) {
        let settings = AppSettings {
            theme: Some(self.theme),
            follow_system_theme: Some(self.follow_system_theme),
            saved_crafts: self.saved_crafts.clone(),
            wiki_cached_items: self.wiki_cached_items.clone(),
            wiki_http_etag_cache: self.wiki_http_etag_cache.clone(),
            wiki_http_last_modified_cache: self.wiki_http_last_modified_cache.clone(),
            wiki_last_sync_unix_seconds: self.wiki_last_sync_unix_seconds,
        };

        if let Ok(raw) = serde_json::to_string(&settings) {
            storage.set_string(APP_SETTINGS_KEY, raw);
        }
    }
}

impl Default for MdcraftApp {
    fn default() -> Self {
        let system_theme = detect_system_theme();
        let wiki_cached_items = embedded_wiki_items();
        let resource_list = embedded_resource_names();

        Self {
            input_text: String::new(),
            items: Vec::new(),
            sell_price_input: String::new(),
            resource_list,
            fonts_loaded: false,
            theme: system_theme,
            follow_system_theme: true,
            sidebar_open: true,
            saved_crafts: Vec::new(),
            pending_craft_name: String::new(),
            awaiting_craft_name: false,
            focus_craft_name_input: false,
            pending_delete_index: None,
            active_saved_craft_index: None,
            awaiting_import_json: false,
            import_json_input: String::new(),
            import_feedback: None,
            awaiting_export_json: false,
            export_json_output: String::new(),
            export_feedback: None,
            wiki_sync_feedback: None,
            wiki_cached_items,
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_refresh_in_progress: false,
            wiki_refresh_rx: None,
            wiki_sync_success_anim_started_at: None,
            wiki_refresh_started_on_launch: false,
            wiki_last_sync_unix_seconds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::{CreationContext, Storage, egui};

    use super::{APP_SETTINGS_KEY, AppSettings, MdcraftApp, SavedCraft, Theme, detect_system_theme};
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
            wiki_http_etag_cache: HashMap::from([(String::from("https://wiki"), String::from("etag1"))]),
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
        assert_eq!(app.wiki_http_etag_cache.get("https://wiki").map(String::as_str), Some("etag1"));
        assert_eq!(
            app.wiki_http_last_modified_cache
                .get("https://wiki")
                .map(String::as_str),
            Some("Wed, 21 Oct 2015 07:28:00 GMT")
        );
        assert_eq!(app.wiki_last_sync_unix_seconds, Some(1_700_000_000));
    }
}
