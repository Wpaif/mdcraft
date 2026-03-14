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

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn toggle_switches_between_light_and_dark() {
        assert_eq!(Theme::Light.toggle(), Theme::Dark);
        assert_eq!(Theme::Dark.toggle(), Theme::Light);
    }

    #[test]
    fn visuals_match_expected_mode() {
        assert!(!Theme::Light.visuals().dark_mode);
        assert!(Theme::Dark.visuals().dark_mode);
    }
}
