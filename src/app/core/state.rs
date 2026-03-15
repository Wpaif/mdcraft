use crate::app::ui_sections::craft_input::local_search_thread::{
    LocalSearchMsg, LocalSearchResult,
};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use crate::data::wiki_scraper::{
    ScrapeRefreshData, ScrapedCraftRecipe, ScrapedItem, embedded_craft_recipes,
    embedded_resource_names, embedded_wiki_items,
};
use crate::model::Item;

use super::{SavedCraft, Theme, build_craft_recipe_name_index, detect_system_theme};

/// The application state that is passed to `eframe`.
///
/// In GKT4 terms, this is the *model* for the main window; the view logic lives
/// in `ui.rs` and helpers are in other submodules.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecipeSavePopupType {
    Save,
    Update,
}

pub struct MdcraftApp {
    pub show_recipe_save_popup: Option<RecipeSavePopupType>,
    /// Nome do craft selecionado pelo usuário (editável)
    pub selected_craft_name: String,
    pub es_suggestions: Vec<String>,
    pub es_error: Option<String>,
    pub es_query_tx: Option<Sender<LocalSearchMsg>>,
    pub es_result_rx: Option<Receiver<LocalSearchResult>>,
    pub craft_search_query: String,
    pub craft_search_qty: u64,
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
    /// Marca o instante de erro/interrupção na sync da wiki (para cooldown visual)
    pub wiki_sync_error_anim_started_at: Option<Instant>,
    pub wiki_cached_items: Vec<ScrapedItem>,
    pub craft_recipes_cache: Vec<ScrapedCraftRecipe>,
    pub craft_recipe_name_by_signature: HashMap<String, String>,
    pub wiki_http_etag_cache: HashMap<String, String>,
    pub wiki_http_last_modified_cache: HashMap<String, String>,
    pub wiki_refresh_in_progress: bool,
    pub wiki_refresh_rx: Option<Receiver<Result<ScrapeRefreshData, String>>>,
    pub wiki_sync_success_anim_started_at: Option<Instant>,
    pub wiki_refresh_started_on_launch: bool,
    pub wiki_last_sync_unix_seconds: Option<u64>,
    pub recipe_save_toast_started_at: Option<Instant>,
    pub last_saved_recipe_name: Option<String>,
}

impl Default for MdcraftApp {
    fn default() -> Self {
        let system_theme = detect_system_theme();
        let wiki_cached_items = embedded_wiki_items();
        let craft_recipes_cache = embedded_craft_recipes();
        let craft_recipe_name_by_signature = build_craft_recipe_name_index(&craft_recipes_cache);
        let resource_list = embedded_resource_names();

        let (es_query_tx, es_result_rx) =
            crate::app::ui_sections::craft_input::local_search_thread::start_local_search_thread();

        Self {
            show_recipe_save_popup: None,
            selected_craft_name: String::new(),
            craft_search_query: String::new(),
            craft_search_qty: 1,
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
            wiki_sync_error_anim_started_at: None,
            wiki_cached_items,
            craft_recipes_cache,
            craft_recipe_name_by_signature,
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_refresh_in_progress: false,
            wiki_refresh_rx: None,
            wiki_sync_success_anim_started_at: None,
            wiki_refresh_started_on_launch: false,
            wiki_last_sync_unix_seconds: None,
            recipe_save_toast_started_at: None,
            last_saved_recipe_name: None,
            es_suggestions: Vec::new(),
            es_error: None,
            es_query_tx: Some(es_query_tx),
            es_result_rx: Some(es_result_rx),
        }
    }
}
