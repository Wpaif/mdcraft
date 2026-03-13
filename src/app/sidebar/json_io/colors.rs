use eframe::egui;

pub(super) fn action_button_colors(ui: &egui::Ui) -> (egui::Color32, egui::Stroke, egui::Color32) {
    let is_dark = ui.visuals().dark_mode;
    let fill = if is_dark {
        egui::Color32::from_rgb(56, 98, 74)
    } else {
        egui::Color32::from_rgb(101, 144, 116)
    };
    let stroke = if is_dark {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 173, 138))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(78, 120, 95))
    };
    let text = if is_dark {
        egui::Color32::from_rgb(242, 248, 241)
    } else {
        egui::Color32::from_rgb(245, 250, 244)
    };
    (fill, stroke, text)
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::action_button_colors;

    #[test]
    fn action_button_colors_returns_non_zero_stroke_and_alpha() {
        egui::__run_test_ui(|ui| {
            let (fill, stroke, _text) = action_button_colors(ui);
            assert!(fill.a() > 0);
            assert!(stroke.width > 0.0);
        });
    }
}
