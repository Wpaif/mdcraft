use std::cell::RefCell;
use std::collections::HashMap;

use eframe::CreationContext;
use eframe::Storage;

use crate::app::{MdcraftApp, SavedCraft, Theme};

use super::{APP_SETTINGS_KEY, AppSettings, SavedCraftStore};

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
        sell_price_is_per_item: false,
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
        sell_price_is_per_item: false,
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
