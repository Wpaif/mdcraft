use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{MdcraftApp, SavedCraft, Theme};
use crate::data::wiki_scraper::ScrapedItem;

pub(crate) const APP_SETTINGS_KEY: &str = "mdcraft.app_settings";

pub(crate) trait SavedCraftStore {
    fn load_saved_crafts(&self) -> Result<Vec<SavedCraft>, String>;
    fn save_saved_crafts(&self, saved_crafts: &[SavedCraft]) -> Result<(), String>;
}

struct SqliteSavedCraftStore;

impl SavedCraftStore for SqliteSavedCraftStore {
    fn load_saved_crafts(&self) -> Result<Vec<SavedCraft>, String> {
        super::sqlite::load_saved_crafts_from_sqlite()
    }

    fn save_saved_crafts(&self, saved_crafts: &[SavedCraft]) -> Result<(), String> {
        super::sqlite::save_saved_crafts_to_sqlite(saved_crafts)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub(super) struct AppSettings {
    pub(super) theme: Option<Theme>,
    pub(super) follow_system_theme: Option<bool>,
    #[serde(default)]
    pub(super) saved_crafts: Vec<SavedCraft>,
    #[serde(default)]
    pub(super) wiki_cached_items: Vec<ScrapedItem>,
    #[serde(default)]
    pub(super) wiki_http_etag_cache: HashMap<String, String>,
    #[serde(default)]
    pub(super) wiki_http_last_modified_cache: HashMap<String, String>,
    pub(super) wiki_last_sync_unix_seconds: Option<u64>,
}

impl MdcraftApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        Self::from_creation_context_with_store(cc, &SqliteSavedCraftStore, !cfg!(test))
    }

    fn from_creation_context_with_store(
        cc: &eframe::CreationContext<'_>,
        store: &dyn SavedCraftStore,
        load_from_store: bool,
    ) -> Self {
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

        // Keep tests deterministic by avoiding local SQLite state leakage.
        if load_from_store {
            if let Ok(saved_crafts) = store.load_saved_crafts() {
                app.saved_crafts = saved_crafts;
            }
        }

        app
    }

    pub(crate) fn persist_saved_crafts_to_sqlite(&self) {
        self.persist_saved_crafts_with_store(&SqliteSavedCraftStore);
    }

    fn persist_saved_crafts_with_store(&self, store: &dyn SavedCraftStore) {
        if let Err(err) = store.save_saved_crafts(&self.saved_crafts) {
            eprintln!("Falha ao persistir receitas no SQLite: {err}");
        }
    }

    pub fn save_app_settings(&self, storage: &mut dyn eframe::Storage) {
        self.save_app_settings_with_store(storage, &SqliteSavedCraftStore);
    }

    fn save_app_settings_with_store(
        &self,
        storage: &mut dyn eframe::Storage,
        store: &dyn SavedCraftStore,
    ) {
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

        self.persist_saved_crafts_with_store(store);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use eframe::{CreationContext, Storage};

    use super::{APP_SETTINGS_KEY, AppSettings, MdcraftApp, SavedCraft, SavedCraftStore, Theme};

    #[derive(Default)]
    struct FakeStore {
        loaded: RefCell<Vec<SavedCraft>>,
        saved_snapshots: RefCell<Vec<Vec<SavedCraft>>>,
    }

    impl SavedCraftStore for FakeStore {
        fn load_saved_crafts(&self) -> Result<Vec<SavedCraft>, String> {
            Ok(self.loaded.borrow().clone())
        }

        fn save_saved_crafts(&self, saved_crafts: &[SavedCraft]) -> Result<(), String> {
            self.saved_snapshots
                .borrow_mut()
                .push(saved_crafts.to_vec());
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_string(), value);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn from_creation_context_with_store_loads_saved_crafts_from_store() {
        let store = FakeStore::default();
        store.loaded.borrow_mut().push(SavedCraft {
            name: "From Store".to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "10k".to_string(),
            item_prices: vec![],
        });

        let mut cc = CreationContext::_new_kittest(eframe::egui::Context::default());
        cc.storage = None;

        let app = MdcraftApp::from_creation_context_with_store(&cc, &store, true);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.saved_crafts[0].name, "From Store");
    }

    #[test]
    fn save_app_settings_with_store_persists_to_storage_and_store() {
        let store = FakeStore::default();
        let mut app = MdcraftApp::default();
        app.theme = Theme::Dark;
        app.saved_crafts.push(SavedCraft {
            name: "Persist Me".to_string(),
            recipe_text: "2 Screw".to_string(),
            sell_price_input: "5k".to_string(),
            item_prices: vec![],
        });

        let mut storage = MemoryStorage::default();
        app.save_app_settings_with_store(&mut storage, &store);

        let raw = storage
            .get_string(APP_SETTINGS_KEY)
            .expect("settings payload should be written");
        let parsed: AppSettings =
            serde_json::from_str(&raw).expect("settings payload should deserialize");
        assert_eq!(parsed.theme, Some(Theme::Dark));
        assert_eq!(parsed.saved_crafts.len(), 1);

        let snapshots = store.saved_snapshots.borrow();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0][0].name, "Persist Me");
    }
}
