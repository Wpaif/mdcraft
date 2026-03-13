use std::collections::HashMap;

use crate::data::wiki_scraper::ScrapedCraftRecipe;
use crate::model::Item;

use super::normalize::normalized_ingredient_key;

fn compose_craft_signature(mut entries: Vec<(String, u64)>) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let joined = entries
        .into_iter()
        .map(|(name, qty)| format!("{name}:{qty}"))
        .collect::<Vec<_>>()
        .join("|");

    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn recipe_quantity_to_u64(quantity: f64) -> Option<u64> {
    let rounded = quantity.round();
    if (quantity - rounded).abs() > 1e-6 || rounded <= 0.0 {
        return None;
    }

    Some(rounded as u64)
}

pub(super) fn ingredient_quantities_from_items(items: &[Item]) -> HashMap<String, u64> {
    let mut per_item = HashMap::<String, u64>::new();

    for item in items {
        let key = normalized_ingredient_key(&item.nome);
        if key.is_empty() {
            continue;
        }
        *per_item.entry(key).or_insert(0) += item.quantidade;
    }

    per_item
}

pub(super) fn ingredient_quantities_from_recipe(
    recipe: &ScrapedCraftRecipe,
) -> Option<HashMap<String, u64>> {
    let mut per_item = HashMap::<String, u64>::new();

    for ingredient in &recipe.ingredients {
        let qty = recipe_quantity_to_u64(ingredient.quantity)?;
        let key = normalized_ingredient_key(&ingredient.name);
        if key.is_empty() {
            continue;
        }
        *per_item.entry(key).or_insert(0) += qty;
    }

    Some(per_item)
}

pub(super) fn is_recipe_multiple_of_items(
    item_quantities: &HashMap<String, u64>,
    recipe_quantities: &HashMap<String, u64>,
) -> bool {
    if item_quantities.len() != recipe_quantities.len() {
        return false;
    }

    let mut multiplier: Option<u64> = None;

    for (name, recipe_qty) in recipe_quantities {
        if *recipe_qty == 0 {
            return false;
        }

        let Some(item_qty) = item_quantities.get(name).copied() else {
            return false;
        };

        if item_qty == 0 || item_qty % recipe_qty != 0 {
            return false;
        }

        let current_multiplier = item_qty / recipe_qty;
        if current_multiplier == 0 {
            return false;
        }

        match multiplier {
            Some(existing) if existing != current_multiplier => return false,
            None => multiplier = Some(current_multiplier),
            _ => {}
        }
    }

    multiplier.is_some()
}

pub(super) fn craft_signature_from_items(items: &[Item]) -> Option<String> {
    compose_craft_signature(ingredient_quantities_from_items(items).into_iter().collect())
}

pub(super) fn craft_signature_from_recipe(recipe: &ScrapedCraftRecipe) -> Option<String> {
    compose_craft_signature(ingredient_quantities_from_recipe(recipe)?.into_iter().collect())
}
