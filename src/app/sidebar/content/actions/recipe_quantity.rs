use std::collections::HashMap;

pub(super) fn normalize_item_key(name: &str) -> String {
    name.split_whitespace()
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(super) fn quantities_from_recipe_text(recipe_text: &str) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    for part in recipe_text.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut it = part.split_whitespace();
        let Some(qty_str) = it.next() else {
            continue;
        };
        let Ok(qty) = qty_str.parse::<u64>() else {
            continue;
        };
        let name = it.collect::<Vec<_>>().join(" ");
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        out.insert(normalize_item_key(name), qty);
    }
    out
}

pub(super) fn infer_craft_count_from_saved_recipe(
    craft: &crate::app::SavedCraft,
    recipe: &crate::data::wiki_scraper::ScrapedCraftRecipe,
) -> Option<u64> {
    let qty_map = quantities_from_recipe_text(&craft.recipe_text);
    let mut inferred: Option<u64> = None;

    for ing in &recipe.ingredients {
        let base_qty = ing.quantity as u64;
        if base_qty == 0 {
            continue;
        }
        let Some(saved_qty) = qty_map.get(&normalize_item_key(&ing.name)) else {
            continue;
        };
        if *saved_qty < base_qty || *saved_qty % base_qty != 0 {
            continue;
        }
        let ratio = *saved_qty / base_qty;
        if ratio == 0 {
            continue;
        }
        match inferred {
            None => inferred = Some(ratio),
            Some(existing) if existing == ratio => {}
            Some(_) => return None,
        }
    }

    inferred
}

pub(super) fn display_name_from_normalized_key(name_key: &str) -> String {
    name_key
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

