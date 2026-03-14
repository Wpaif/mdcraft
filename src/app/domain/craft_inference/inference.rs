use std::collections::HashMap;

use crate::data::wiki_scraper::ScrapedCraftRecipe;
use crate::model::Item;

use super::fuzzy::{build_ingredient_vocabulary, fuzzy_resolve_ingredient};
use super::quantities::{
    craft_signature_from_items, ingredient_quantities_from_items,
    ingredient_quantities_from_recipe, is_recipe_multiple_of_items,
};

pub(super) fn infer_craft_name_from_items(
    items: &[Item],
    recipes: &[ScrapedCraftRecipe],
    recipe_name_by_signature: &HashMap<String, String>,
) -> Option<String> {
    let item_signature = craft_signature_from_items(items)?;
    if let Some(exact_name) = recipe_name_by_signature.get(&item_signature) {
        return Some(exact_name.clone());
    }

    let vocab = build_ingredient_vocabulary(recipes);
    let resolved: Vec<Item> = items
        .iter()
        .map(|item| {
            let canonical = fuzzy_resolve_ingredient(&item.nome, &vocab);
            let mut resolved = item.clone();
            resolved.nome = canonical.to_string();
            resolved
        })
        .collect();

    if let Some(resolved_sig) = craft_signature_from_items(&resolved) {
        if resolved_sig != item_signature {
            if let Some(name) = recipe_name_by_signature.get(&resolved_sig) {
                return Some(name.clone());
            }
        }
    }

    let item_quantities = ingredient_quantities_from_items(&resolved);
    if item_quantities.is_empty() {
        return None;
    }

    let mut matched_name: Option<&str> = None;

    for recipe in recipes {
        let Some(recipe_quantities) = ingredient_quantities_from_recipe(recipe) else {
            continue;
        };

        if !is_recipe_multiple_of_items(&item_quantities, &recipe_quantities) {
            continue;
        }

        match matched_name {
            Some(current) if recipe.name.as_str() >= current => {}
            _ => matched_name = Some(recipe.name.as_str()),
        }
    }

    matched_name.map(str::to_owned)
}
