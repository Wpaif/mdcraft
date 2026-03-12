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

// Helper functions used by the theme toggle button UI.

pub fn paint_theme_icon(p: &egui::Painter, rect: egui::Rect, theme: Theme, color: egui::Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.26;

    match theme {
        Theme::Light => {
            // Sun: filled circle + rays
            p.circle_filled(center, r, color);
            let ray_color = color;
            let ray_len = r * 1.9;
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = egui::vec2(a.cos(), a.sin());
                let a0 = center + dir * (r * 1.2);
                let a1 = center + dir * (ray_len * 0.65);
                p.line_segment([a0, a1], egui::Stroke::new(1.5, ray_color));
            }
        }
        Theme::Dark => {
            // Moon: circle - offset circle (crescent)
            p.circle_filled(center, r, color);
            let cut = center + egui::vec2(r * 0.55, -r * 0.15);
            p.circle_filled(cut, r * 0.95, egui::Color32::TRANSPARENT);
        }
    }
}

pub fn theme_toggle_button(ui: &mut egui::Ui, theme: Theme) -> egui::Response {
    let size = egui::vec2(36.0, 36.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rounding = egui::CornerRadius::same(18);
        let fill = visuals.bg_fill;
        let stroke = visuals.bg_stroke;

        let p = ui.painter();
        p.rect(rect, rounding, fill, stroke, egui::StrokeKind::Inside);

        // Icon color: use fg stroke color for maximum contrast.
        let icon_color = visuals.fg_stroke.color;
        let icon_rect = rect.shrink(6.0);

        match theme {
            Theme::Light => {
                paint_theme_icon(p, icon_rect, Theme::Light, icon_color);
            }
            Theme::Dark => {
                // Moon crescent: paint moon, then "cut" with fill color.
                let center = icon_rect.center();
                let r = icon_rect.width().min(icon_rect.height()) * 0.26;
                p.circle_filled(center, r, icon_color);
                let cut = center + egui::vec2(r * 0.55, -r * 0.15);
                p.circle_filled(cut, r * 0.95, fill);
            }
        }
    }

    response
}
