use super::SavedCraft;

pub(crate) const APP_SETTINGS_KEY: &str = "mdcraft.app_settings";

#[path = "settings/app_ops.rs"]
mod app_ops;
#[path = "settings/model.rs"]
mod model;
#[cfg(test)]
#[path = "settings/tests.rs"]
mod tests;

pub(super) use model::AppSettings;

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
