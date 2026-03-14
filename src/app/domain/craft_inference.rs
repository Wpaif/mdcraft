//! Craft recipe name inference and ingredient matching.
//!
//! Provides fuzzy resolution of ingredient names, signature-based recipe
//! look-up, and the index builder used by `MdcraftApp`.

use std::collections::HashMap;

use crate::data::wiki_scraper::ScrapedCraftRecipe;
use crate::model::Item;

#[path = "craft_inference/fuzzy.rs"]
mod fuzzy;
#[path = "craft_inference/index.rs"]
mod index;
#[path = "craft_inference/inference.rs"]
mod inference;
#[path = "craft_inference/normalize.rs"]
mod normalize;
#[path = "craft_inference/quantities.rs"]
mod quantities;
#[cfg(test)]
#[path = "craft_inference/tests.rs"]
mod tests;

pub(super) fn normalized_item_key(name: &str) -> String {
    normalize::normalized_item_key(name)
}

pub(crate) fn infer_craft_name_from_items(
    items: &[Item],
    recipes: &[ScrapedCraftRecipe],
    recipe_name_by_signature: &HashMap<String, String>,
) -> Option<String> {
    inference::infer_craft_name_from_items(items, recipes, recipe_name_by_signature)
}

pub(crate) fn build_craft_recipe_name_index(
    recipes: &[ScrapedCraftRecipe],
) -> HashMap<String, String> {
    index::build_craft_recipe_name_index(recipes)
}
