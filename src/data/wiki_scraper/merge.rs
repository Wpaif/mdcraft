use std::collections::HashMap;

use super::{ScrapedItem, items_parser};

fn has_npc_price(item: &ScrapedItem) -> bool {
    item.npc_price
        .as_deref()
        .map(|p| !p.trim().is_empty())
        .unwrap_or(false)
}

// Filtro removido: agora todos os itens são mantidos, mesmo sem preço NPC
pub(super) fn finalize_scraped_items(mut items: Vec<ScrapedItem>) -> Vec<ScrapedItem> {
    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

pub(super) fn build_existing_price_map(existing: &[ScrapedItem]) -> HashMap<String, String> {
    existing
        .iter()
        .filter_map(|item| {
            item.npc_price
                .as_ref()
                .map(|price| (items_parser::normalize_key(&item.name), price.clone()))
        })
        .collect()
}

pub(super) fn normalized_resource_names(items: &[ScrapedItem]) -> Vec<String> {
    let mut names: Vec<String> = items
        .iter()
        .map(|item| item.name.trim().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect();

    names.sort();
    names.dedup();
    names
}

pub(super) fn merge_items(
    merged: &mut HashMap<String, ScrapedItem>,
    new_items: impl IntoIterator<Item = ScrapedItem>,
) {
    for item in new_items {
        let key = items_parser::normalize_key(&item.name);

        if let Some(existing) = merged.get_mut(&key) {
            if existing.npc_price.is_none()
                && let Some(price) = item.npc_price {
                existing.npc_price = Some(price);
            }
            for source in item.sources {
                if !existing.sources.contains(&source) {
                    existing.sources.push(source);
                }
            }
        } else {
            merged.insert(key, item);
        }
    }
}
