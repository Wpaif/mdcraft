use eframe::egui;
use serde::{Deserialize, Serialize};

// The `Theme` enum represents the application's chosen color theme (light or dark).
// It is separate from the top-level `theme` module which provides color palettes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn visuals(self) -> egui::Visuals {
        match self {
            Theme::Light => crate::theme::github_light(),
            Theme::Dark => crate::theme::doki_dark(),
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}
