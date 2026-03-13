use eframe::egui;

fn hex(rgb: u32) -> egui::Color32 {
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    egui::Color32::from_rgb(r, g, b)
}

pub fn github_light() -> egui::Visuals {
    // Adwaita Light palette — inspired by GNOME/Nautilus light mode.
    let canvas = hex(0xffffff); // main view bg (window_fill)
    let bg = hex(0xf6f5f4);     // sidebar/panel bg
    let overlay = hex(0xedeceb); // hover fill, faint bg
    let border = hex(0xd4d0cb);  // borders / separators
    let text = hex(0x2d2d2d);    // primary text
    let muted = hex(0x706b65);   // secondary text (exact Adwaita value)
    let accent = hex(0x3584e4);  // GNOME blue
    let good = hex(0x2ec27e);    // success green
    let danger = hex(0xe01b24);  // error red
    let warn = hex(0xe5a50a);    // warning amber

    let mut v = egui::Visuals::light();

    v.override_text_color = Some(text);
    v.hyperlink_color = accent;
    v.faint_bg_color = overlay;
    v.extreme_bg_color = canvas;
    v.code_bg_color = overlay;
    v.window_fill = canvas;
    v.panel_fill = bg;

    v.selection.bg_fill = accent;
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    v.widgets.noninteractive.bg_fill = canvas;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, muted);

    v.widgets.inactive.bg_fill = canvas;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.hovered.bg_fill = overlay;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, accent);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.active.bg_fill = overlay;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, good);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.open.bg_fill = overlay;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent);
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    v.error_fg_color = danger;
    v.warn_fg_color = warn;

    v
}

pub fn doki_dark() -> egui::Visuals {
    // Adwaita Dark palette — inspired by GNOME/Nautilus dark mode.
    let base = hex(0x2d2d2d);    // sidebar chrome (panel_fill)
    let surface = hex(0x242424); // main view bg (window_fill)
    let deep = hex(0x1e1e1e);    // deepest bg: text inputs, extreme bg
    let overlay = hex(0x3a3a3a); // hover fill, faint bg
    let border = hex(0x4a4a4a);  // borders / separators
    let text = hex(0xffffff);    // primary text
    let muted = hex(0x9a9996);   // secondary text (exact Adwaita value)
    let accent = hex(0x3584e4);  // GNOME blue
    let accent2 = hex(0x5ab0f6); // hyperlink — lighter blue
    let good = hex(0x2ec27e);    // success green
    let danger = hex(0xe01b24);  // error red
    let warn = hex(0xe5a50a);    // warning amber

    let mut v = egui::Visuals::dark();

    v.override_text_color = Some(text);
    v.hyperlink_color = accent2;
    v.faint_bg_color = overlay;
    v.extreme_bg_color = deep;
    v.code_bg_color = overlay;
    v.window_fill = surface;
    v.panel_fill = base;

    v.selection.bg_fill = accent;
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    v.widgets.noninteractive.bg_fill = base;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, muted);

    v.widgets.inactive.bg_fill = surface;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.hovered.bg_fill = overlay;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, accent);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.active.bg_fill = overlay;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, good);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.open.bg_fill = surface;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent);
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    v.error_fg_color = danger;
    v.warn_fg_color = warn;

    v
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{doki_dark, github_light, hex};

    #[test]
    fn hex_splits_rgb_channels_correctly() {
        let c = hex(0x12_ab_ef);
        assert_eq!(c, egui::Color32::from_rgb(0x12, 0xab, 0xef));
    }

    #[test]
    fn github_light_uses_expected_palette_points() {
        let v = github_light();
        assert_eq!(v.dark_mode, false);
        assert_eq!(v.panel_fill, egui::Color32::from_rgb(0xf6, 0xf5, 0xf4));
        assert_eq!(v.hyperlink_color, egui::Color32::from_rgb(0x35, 0x84, 0xe4));
        assert_eq!(v.error_fg_color, egui::Color32::from_rgb(0xe0, 0x1b, 0x24));
    }

    #[test]
    fn doki_dark_uses_expected_palette_points() {
        let v = doki_dark();
        assert_eq!(v.dark_mode, true);
        assert_eq!(v.panel_fill, egui::Color32::from_rgb(0x2d, 0x2d, 0x2d));
        assert_eq!(v.hyperlink_color, egui::Color32::from_rgb(0x5a, 0xb0, 0xf6));
        assert_eq!(v.error_fg_color, egui::Color32::from_rgb(0xe0, 0x1b, 0x24));
    }
}
