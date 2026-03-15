/// Função genérica para popups/modais customizáveis
pub fn render_modal_window<R>(
    ctx: &egui::Context,
    id: egui::Id,
    title: &str,
    fixed_size: egui::Vec2,
    anchor: egui::Align2,
    offset: egui::Vec2,
    bg_color: egui::Color32,
    border_color: egui::Color32,
    title_color: egui::Color32,
    border_radius: f32,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let mut result = None;
    egui::Window::new(title)
        .id(id)
        .anchor(anchor, offset)
        .collapsible(false)
        .resizable(false)
        .fixed_size(fixed_size)
        .frame(
            egui::Frame::window(
                &ctx.style()
            )
            .fill(bg_color)
            .stroke(egui::Stroke::new(2.0, border_color))
            .corner_radius(border_radius)
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .color(title_color)
                        .size(18.0)
                );
            });
            ui.add_space(8.0);
            result = Some(content(ui));
        });
    result
}
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

/// Toast genérico customizável, inspirado no visual do toast de sincronização
pub fn render_toast_area(
    ctx: &egui::Context,
    id: egui::Id,
    main_text: &str,
    sub_text: Option<&str>,
    bg_color: egui::Color32,
    border_color: egui::Color32,
    sub_color: egui::Color32,
    started_at: std::time::Instant,
    duration: std::time::Duration,
) -> bool {
    let elapsed = started_at.elapsed();
    if elapsed >= duration {
        return true; // Toast terminou
    }
    let t = (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0);
    let fade_in = (t / 0.18).clamp(0.0, 1.0);
    let fade_out = ((1.0 - t) / 0.24).clamp(0.0, 1.0);
    let alpha = (fade_in * fade_out).clamp(0.0, 1.0);
    let y_offset = (1.0 - fade_in) * 14.0;

    // Cores coesas e modernas
    let bg = egui::Color32::from_rgba_unmultiplied(
        (bg_color.r() as f32 * 0.85) as u8,
        (bg_color.g() as f32 * 0.85) as u8,
        (bg_color.b() as f32 * 0.85) as u8,
        (220.0 * alpha) as u8,
    );
    let border = egui::Color32::from_rgba_unmultiplied(
        (border_color.r() as f32 * 0.95) as u8,
        (border_color.g() as f32 * 0.95) as u8,
        (border_color.b() as f32 * 0.95) as u8,
        (180.0 * alpha) as u8,
    );
    let shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, (60.0 * alpha) as u8),
    };

    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-22.0, -22.0 + y_offset))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(bg)
                .stroke(egui::Stroke::new(1.5, border))
                .shadow(shadow)
                .corner_radius(12.0)
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(main_text)
                            .strong()
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                    if let Some(sub) = sub_text {
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(sub)
                                .size(13.0)
                                .color(sub_color.linear_multiply(0.92)),
                        );
                    }
                });
        });
    ctx.request_repaint();
    false // Toast ainda ativo
}
