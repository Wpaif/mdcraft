use crate::parse::parse_price_flag;

use super::super::super::price::PriceStatus;

pub fn apply_item_price_from_input(item: &mut crate::model::Item) {
    item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0.0);
    item.valor_total = item.preco_unitario * item.quantidade as f64;
}

pub(super) fn apply_item_price_if_changed(item: &mut crate::model::Item, price_changed: bool) {
    if price_changed {
        apply_item_price_from_input(item);
    }
}

pub(super) fn item_price_status(item: &crate::model::Item) -> PriceStatus {
    if !item.preco_input.is_empty() && parse_price_flag(&item.preco_input).is_err() {
        PriceStatus::Invalid
    } else if item.valor_total > 0.0 {
        PriceStatus::Ok
    } else {
        PriceStatus::None
    }
}

pub(super) fn item_status_hover(status: PriceStatus) -> Option<&'static str> {
    match status {
        PriceStatus::Invalid => Some("Valor Inválido"),
        PriceStatus::Ok => Some("OK"),
        PriceStatus::None => None,
    }
}
