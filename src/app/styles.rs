use eframe::egui;

/// UI style helpers that are specific to this application rather than a general theme.

pub fn setup_custom_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(26.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(20);
    style.spacing.interact_size = egui::vec2(40.0, 24.0);
    style.spacing.text_edit_width = 150.0;

    ctx.set_style(style);
}

pub fn setup_emoji_support(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "emoji".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/NotoEmoji-Regular.ttf")).into(),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("emoji".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("emoji".to_owned());

    ctx.set_fonts(fonts);
}
