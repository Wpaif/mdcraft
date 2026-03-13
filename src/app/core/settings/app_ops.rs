use crate::app::MdcraftApp;

use super::APP_SETTINGS_KEY;
use super::AppSettings;
use super::SavedCraftStore;
use super::SqliteSavedCraftStore;

impl MdcraftApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        Self::from_creation_context_with_store(cc, &SqliteSavedCraftStore, !cfg!(test))
    }

    pub(in crate::app::settings) fn from_creation_context_with_store(
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

    pub(in crate::app::settings) fn save_app_settings_with_store(
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
