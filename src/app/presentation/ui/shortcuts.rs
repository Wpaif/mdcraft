use eframe::egui;

pub(super) fn apply_sidebar_toggle_shortcut(app: &mut crate::app::MdcraftApp, ctx: &egui::Context) {
    let toggle_pressed = ctx.input_mut(|i| {
        i.consume_shortcut(&egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL,
            egui::Key::E,
        ))
    });
    if toggle_pressed {
        app.sidebar_open = !app.sidebar_open;
    }
}
