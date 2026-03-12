use eframe::egui;

fn hex(rgb: u32) -> egui::Color32 {
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    egui::Color32::from_rgb(r, g, b)
}

pub fn github_light() -> egui::Visuals {
    // Approximation of GitHub Light (primer) tuned for egui.
    let canvas = hex(0xffffff);
    let bg = hex(0xf6f8fa);
    let overlay = hex(0xe1e4e8);
    let border = hex(0xc7ced6);
    let text = hex(0x24292f);
    let muted = hex(0x57606a);
    let accent = hex(0x0969da);
    let success = hex(0x1a7f37);
    let danger = hex(0xd1242f);
    let attention = hex(0x9a6700);

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

    // Make groups/popups pop against the panel background:
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
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, success);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.open.bg_fill = overlay;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent);
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    v.error_fg_color = danger;
    v.warn_fg_color = attention;

    v
}

pub fn doki_dark() -> egui::Visuals {
    // "Doki Theme" inspired dark palette (high contrast + vibrant accents) tuned for egui.
    let base = hex(0x1c1d26);
    let surface = hex(0x232433);
    let overlay = hex(0x2e2f3e);
    let border = hex(0x3a3b4d);
    let text = hex(0xe6e6e6);
    let muted = hex(0xa1a1aa);
    let accent = hex(0xff79c6);
    let accent2 = hex(0x8be9fd);
    let good = hex(0x50fa7b);
    let danger = hex(0xff5555);
    let warn = hex(0xf1fa8c);

    let mut v = egui::Visuals::dark();

    v.override_text_color = Some(text);
    v.hyperlink_color = accent2;
    v.faint_bg_color = overlay;
    v.extreme_bg_color = base;
    v.code_bg_color = overlay;
    v.window_fill = surface;
    v.panel_fill = base;

    v.selection.bg_fill = accent;
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

    v.widgets.noninteractive.bg_fill = base;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, muted);

    v.widgets.inactive.bg_fill = surface;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);

    v.widgets.hovered.bg_fill = overlay;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, accent2);
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
