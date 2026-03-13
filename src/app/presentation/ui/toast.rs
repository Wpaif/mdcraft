use std::time::Duration;

use eframe::egui;

pub(super) fn render_wiki_sync_success_toast(
    app: &mut crate::app::MdcraftApp,
    ctx: &egui::Context,
) {
    let Some(started_at) = app.wiki_sync_success_anim_started_at else {
        return;
    };

    let total = Duration::from_millis(2600);
    let elapsed = started_at.elapsed();

    if elapsed >= total {
        app.wiki_sync_success_anim_started_at = None;
        return;
    }

    let t = (elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0);
    let fade_in = (t / 0.18).clamp(0.0, 1.0);
    let fade_out = ((1.0 - t) / 0.24).clamp(0.0, 1.0);
    let alpha = (fade_in * fade_out).clamp(0.0, 1.0);
    let y_offset = (1.0 - fade_in) * 14.0;

    egui::Area::new(egui::Id::new("wiki_sync_success_toast"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-22.0, -22.0 + y_offset))
        .show(ctx, |ui| {
            let bg = egui::Color32::from_rgba_unmultiplied(26, 127, 55, (225.0 * alpha) as u8);
            let stroke = egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(193, 255, 214, (200.0 * alpha) as u8),
            );

            egui::Frame::new()
                .fill(bg)
                .stroke(stroke)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Base de precos atualizada")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new("Dados sincronizados com a wiki")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(228, 255, 237)),
                    );
                });
        });

    ctx.request_repaint();
}
