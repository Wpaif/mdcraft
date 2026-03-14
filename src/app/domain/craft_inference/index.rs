use std::collections::HashMap;

use crate::data::wiki_scraper::ScrapedCraftRecipe;

use super::quantities::craft_signature_from_recipe;

pub(super) fn build_craft_recipe_name_index(
    recipes: &[ScrapedCraftRecipe],
) -> HashMap<String, String> {
    let mut index = HashMap::new();

    for recipe in recipes {
        let Some(signature) = craft_signature_from_recipe(recipe) else {
            continue;
        };

        index
            .entry(signature)
            .and_modify(|current: &mut String| {
                if recipe.name < *current {
                    *current = recipe.name.clone();
                }
            })
            .or_insert_with(|| recipe.name.clone());
    }

    index
}
