//! Top-level application logic and state.
//!
//! The previous `src/app.rs` file has been broken into smaller submodules
//! following a directory-based layout.  Each feature of the UI has its own
//! file:
//!
//! * `theme_state` – theme enum and toggle button logic.
//! * `styles` – custom egui styling and emoji font support.
//! * `price` – price status indicator helpers.
//! * `ui` – the implementation of `eframe::App` and the main view.

use crate::model::Item;

// `Item` is needed for the application state. Parser functions and formatter are
// now imported in `ui.rs` where the UI logic lives.

// re-export the Theme type so callers can refer to `app::Theme`.
pub use theme_state::Theme;

mod theme_state;
mod styles;
mod price;
mod ui;

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
        }
    }
}
