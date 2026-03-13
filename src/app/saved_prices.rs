use std::collections::HashMap;

use crate::model::Item;
use crate::parse::parse_price_flag;

use super::SavedItemPrice;
use super::craft_inference::normalized_item_key;

pub(crate) fn capture_saved_item_prices(items: &[Item]) -> Vec<SavedItemPrice> {
    let mut result: Vec<SavedItemPrice> = items
        .iter()
        .filter_map(|item| {
            let price_input = item.preco_input.trim();
            if price_input.is_empty() {
                return None;
            }

            Some(SavedItemPrice {
                item_name: item.nome.clone(),
                price_input: price_input.to_string(),
            })
        })
        .collect();

    result.sort_by(|a, b| a.item_name.cmp(&b.item_name));
    result
}

pub(crate) fn apply_saved_item_prices(items: &mut [Item], saved_prices: &[SavedItemPrice]) {
    let lookup: HashMap<String, &str> = saved_prices
        .iter()
        .map(|saved| {
            (
                normalized_item_key(&saved.item_name),
                saved.price_input.as_str(),
            )
        })
        .collect();

    for item in items {
        let key = normalized_item_key(&item.nome);
        let Some(saved_input) = lookup.get(&key).copied() else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(saved_input) else {
            continue;
        };

        item.preco_input = saved_input.to_string();
        item.preco_unitario = parsed;
        item.valor_total = parsed * item.quantidade as f64;
    }
}
