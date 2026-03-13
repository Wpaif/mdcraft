use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriceStatus {
    None,
    Ok,
    Invalid,
}

/// Paints a small status indicator for price validity. Returns the response that
/// can be used for hover text.

pub fn paint_price_status(ui: &mut egui::Ui, status: PriceStatus) -> egui::Response {
    let size = egui::vec2(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

    if !ui.is_rect_visible(rect) || status == PriceStatus::None {
        return response;
    }

    let p = ui.painter();
    match status {
        PriceStatus::Ok => {
            let center = rect.center();
            let r = rect.width().min(rect.height()) * 0.45;
            p.circle_filled(center, r, egui::Color32::from_rgb(26, 127, 55));

            // Check mark (two segments).
            let a = center + egui::vec2(-r * 0.45, r * 0.05);
            let b = center + egui::vec2(-r * 0.10, r * 0.38);
            let c = center + egui::vec2(r * 0.55, -r * 0.35);
            p.line_segment([a, b], egui::Stroke::new(2.0, egui::Color32::WHITE));
            p.line_segment([b, c], egui::Stroke::new(2.0, egui::Color32::WHITE));
        }
        PriceStatus::Invalid => {
            let center = rect.center();
            let r = rect.width().min(rect.height()) * 0.5;

            // Triangle warning sign.
            let top = center + egui::vec2(0.0, -r * 0.95);
            let bl = center + egui::vec2(-r * 0.9, r * 0.85);
            let br = center + egui::vec2(r * 0.9, r * 0.85);
            let yellow = egui::Color32::from_rgb(241, 250, 140);
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(154, 103, 0));
            p.add(egui::Shape::convex_polygon(
                vec![top, br, bl],
                yellow,
                stroke,
            ));

            // Exclamation mark.
            let bar_top = center + egui::vec2(0.0, -r * 0.35);
            let bar_bot = center + egui::vec2(0.0, r * 0.25);
            p.line_segment(
                [bar_top, bar_bot],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(36, 41, 47)),
            );
            p.circle_filled(
                center + egui::vec2(0.0, r * 0.48),
                1.6,
                egui::Color32::from_rgb(36, 41, 47),
            );
        }
        PriceStatus::None => {}
    }

    response
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{PriceStatus, paint_price_status};

    fn paint_and_capture_rect(status: PriceStatus) -> egui::Rect {
        let ctx = egui::Context::default();
        let mut rect = None;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = paint_price_status(ui, status);
                rect = Some(response.rect);
            });
        });

        rect.expect("price status should allocate a response rect")
    }

    #[test]
    fn paint_price_status_allocates_expected_size_for_none() {
        let rect = paint_and_capture_rect(PriceStatus::None);
        assert_eq!(rect.size(), egui::vec2(18.0, 18.0));
    }

    #[test]
    fn paint_price_status_allocates_expected_size_for_ok() {
        let rect = paint_and_capture_rect(PriceStatus::Ok);
        assert_eq!(rect.size(), egui::vec2(18.0, 18.0));
    }

    #[test]
    fn paint_price_status_allocates_expected_size_for_invalid() {
        let rect = paint_and_capture_rect(PriceStatus::Invalid);
        assert_eq!(rect.size(), egui::vec2(18.0, 18.0));
    }
}
