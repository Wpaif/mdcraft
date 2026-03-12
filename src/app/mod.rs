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
use serde::{Deserialize, Serialize};

// `Item` is needed for the application state. Parser functions and formatter are
// now imported in `ui.rs` where the UI logic lives.

// re-export the Theme type so callers can refer to `app::Theme`.
pub use theme_state::Theme;

mod price;
mod sidebar;
mod styles;
mod theme_state;
mod ui;
mod ui_sections;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedCraft {
    pub name: String,
    pub recipe_text: String,
    pub sell_price_input: String,
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
}

impl Default for MdcraftApp {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            items: Vec::new(),
            sell_price_input: String::new(),
            resource_list: vec![
                "tech data".to_string(),
                "iron ore".to_string(),
                "iron bar".to_string(),
                "platinum bar".to_string(),
                "platinum ore".to_string(),
                "pure grass".to_string(),
                "minor seed bag".to_string(),
                "condensed grass".to_string(),
                "nature energy".to_string(),
                "major seed bag".to_string(),
                "pure strong grass".to_string(),
                "condensed strong grass".to_string(),
                "strong nature energy".to_string(),
                "darkrai essence".to_string(),
                "dew becker".to_string(),
                "study note".to_string(),
                "log".to_string(),
                "style point".to_string(),
                "refined style point".to_string(),
                "planks".to_string(),
                "refined fashion point".to_string(),
                "oak planks".to_string(),
                "fashion point".to_string(),
                "purpleheart log".to_string(),
                "nightmare style point".to_string(),
                "drawing clipboard".to_string(),
                "Gold Coins".to_string(),
                "Gold Bar".to_string(),
                "Cooking Token".to_string(),
                "Hidden Relic".to_string(),
                "Corrupted Gold Bar".to_string(),
                "Food Bag".to_string(),
                "Strange Gold Bar".to_string(),
            ],
            fonts_loaded: false,
            theme: Theme::Dark,
            sidebar_open: false,
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
        }
    }
}
