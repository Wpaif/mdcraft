use eframe::egui;
use std::collections::HashMap;

use crate::app::{MdcraftApp, fixed_npc_price_input};
use crate::parse::parse_price_flag;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NpcPriceComparison {
    Equal,
    HigherThanNpc,
    LowerThanNpc,
}

pub(super) fn build_npc_price_lookup(app: &MdcraftApp) -> HashMap<String, f64> {
    let mut lookup = HashMap::new();

    for entry in &app.wiki_cached_items {
        let Some(raw_price) = &entry.npc_price else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(raw_price) else {
            continue;
        };

        lookup.insert(entry.name.trim().to_lowercase(), parsed);
    }

    for fixed_name in ["Compressed Nightmare Gems", "Neutral Essence"] {
        if let Some(raw_price) = fixed_npc_price_input(fixed_name)
            && let Ok(parsed) = parse_price_flag(raw_price)
        {
            lookup.insert(fixed_name.trim().to_lowercase(), parsed);
        }
    }

    lookup
}

pub(super) fn compare_item_price_with_npc(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<NpcPriceComparison> {
    let entered = parse_price_flag(&item.preco_input).ok()?;
    let npc_price = npc_lookup.get(&item.nome.trim().to_lowercase()).copied()?;

    let eps = 1e-9;
    if (entered - npc_price).abs() < eps {
        Some(NpcPriceComparison::Equal)
    } else if entered > npc_price {
        Some(NpcPriceComparison::HigherThanNpc)
    } else {
        Some(NpcPriceComparison::LowerThanNpc)
    }
}

pub(super) fn price_input_stroke(
    ui: &egui::Ui,
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> egui::Stroke {
    if item.preco_input.trim().is_empty() {
        // Missing price gets a warm border to draw attention without looking like an error.
        return egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90));
    }

    let default = ui.visuals().widgets.inactive.bg_stroke;

    match compare_item_price_with_npc(item, npc_lookup) {
        Some(NpcPriceComparison::HigherThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(74, 201, 126))
        }
        Some(NpcPriceComparison::LowerThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(220, 98, 98))
        }
        _ => default,
    }
}

pub(super) fn price_input_fill_color(item: &crate::model::Item) -> egui::Color32 {
    if item.preco_input.trim().is_empty() {
        egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
    } else {
        egui::Color32::TRANSPARENT
    }
}

pub(super) fn npc_price_for_item(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<f64> {
    npc_lookup.get(&item.nome.trim().to_lowercase()).copied()
}

pub(super) fn should_show_npc_price_icon(item_name: &str) -> bool {
    !item_name.trim().eq_ignore_ascii_case("diamond")
}

pub(super) fn paint_npc_price_icon(
    ui: &mut egui::Ui,
    has_npc_price: bool,
    is_equal_to_npc: bool,
) -> egui::Response {
    let size = egui::vec2(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter();
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.45;

    let (fill, stroke, text_color) = if !has_npc_price {
        (
            egui::Color32::from_rgba_unmultiplied(120, 120, 120, 24),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 130, 130)),
            egui::Color32::from_rgb(130, 130, 130),
        )
    } else if is_equal_to_npc {
        (
            egui::Color32::from_rgb(29, 155, 240),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(180, 225, 255)),
            egui::Color32::WHITE,
        )
    } else {
        (
            egui::Color32::from_rgba_unmultiplied(29, 155, 240, 36),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(29, 155, 240)),
            egui::Color32::from_rgb(29, 155, 240),
        )
    };

    painter.circle(center, radius, fill, stroke);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "N",
        egui::FontId::proportional(10.0),
        text_color,
    );

    response
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use std::collections::HashMap;

    use crate::app::MdcraftApp;
    use crate::model::Item;

    use super::{
        NpcPriceComparison, build_npc_price_lookup, compare_item_price_with_npc,
        npc_price_for_item, price_input_fill_color, price_input_stroke, should_show_npc_price_icon,
    };
    use crate::data::wiki_scraper::{ScrapedItem, WikiSource};

    fn make_item(nome: &str, quantidade: u64, preco_input: &str) -> Item {
        Item {
            nome: nome.to_string(),
            quantidade,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: false,
            preco_input: preco_input.to_string(),
        }
    }

    #[test]
    fn compare_item_price_with_npc_covers_equal_cheaper_and_expensive() {
        let mut app = MdcraftApp::default();
        app.wiki_cached_items.push(ScrapedItem {
            name: "Screw".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Loot],
        });

        let lookup = build_npc_price_lookup(&app);

        let equal = make_item("Screw", 1, "1k");
        assert_eq!(
            compare_item_price_with_npc(&equal, &lookup),
            Some(NpcPriceComparison::Equal)
        );

        let cheaper = make_item("Screw", 1, "800");
        assert_eq!(
            compare_item_price_with_npc(&cheaper, &lookup),
            Some(NpcPriceComparison::LowerThanNpc)
        );

        let expensive = make_item("Screw", 1, "2k");
        assert_eq!(
            compare_item_price_with_npc(&expensive, &lookup),
            Some(NpcPriceComparison::HigherThanNpc)
        );
    }

    #[test]
    fn build_npc_price_lookup_includes_fixed_compressed_nightmare_gems() {
        let app = MdcraftApp::default();
        let lookup = build_npc_price_lookup(&app);
        assert_eq!(
            lookup.get("compressed nightmare gems").copied(),
            Some(25_000.0)
        );
    }

    #[test]
    fn build_npc_price_lookup_includes_fixed_neutral_essence() {
        let app = MdcraftApp::default();
        let lookup = build_npc_price_lookup(&app);
        assert_eq!(lookup.get("neutral essence").copied(), Some(1000.0));
    }

    #[test]
    fn price_input_stroke_highlights_missing_value() {
        egui::__run_test_ui(|ui| {
            let item = make_item("Screw", 1, "");
            let lookup = HashMap::new();
            let stroke = price_input_stroke(ui, &item, &lookup);
            assert_eq!(
                stroke,
                egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90))
            );
        });
    }

    #[test]
    fn price_input_fill_color_marks_only_missing_value() {
        let missing = make_item("Screw", 1, "");
        assert_eq!(
            price_input_fill_color(&missing),
            egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
        );

        let present = make_item("Screw", 1, "1k");
        assert_eq!(price_input_fill_color(&present), egui::Color32::TRANSPARENT);
    }

    #[test]
    fn npc_price_for_item_returns_lookup_value() {
        let mut app = MdcraftApp::default();
        app.wiki_cached_items.push(ScrapedItem {
            name: "Screw".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Loot],
        });

        let lookup = build_npc_price_lookup(&app);
        let item = make_item("Screw", 1, "");
        assert_eq!(npc_price_for_item(&item, &lookup), Some(1000.0));
    }

    #[test]
    fn should_show_npc_price_icon_hides_only_for_diamond() {
        assert!(!should_show_npc_price_icon("Diamond"));
        assert!(!should_show_npc_price_icon(" diamond "));
        assert!(should_show_npc_price_icon("Screw"));
    }
}
