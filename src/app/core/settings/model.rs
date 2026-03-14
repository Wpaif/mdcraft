use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::app::SavedCraft;
use crate::app::Theme;
use crate::data::wiki_scraper::ScrapedItem;

#[derive(Serialize, Deserialize, Default)]
pub(in crate::app) struct AppSettings {
    pub(in crate::app) theme: Option<Theme>,
    pub(in crate::app) follow_system_theme: Option<bool>,
    #[serde(default)]
    pub(in crate::app) saved_crafts: Vec<SavedCraft>,
    #[serde(default)]
    pub(in crate::app) wiki_cached_items: Vec<ScrapedItem>,
    #[serde(default)]
    pub(in crate::app) wiki_http_etag_cache: HashMap<String, String>,
    #[serde(default)]
    pub(in crate::app) wiki_http_last_modified_cache: HashMap<String, String>,
    pub(in crate::app) wiki_last_sync_unix_seconds: Option<u64>,
}
