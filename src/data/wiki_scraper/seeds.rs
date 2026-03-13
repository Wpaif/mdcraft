use super::{ScrapedCraftRecipe, ScrapedItem, merge};

const EMBEDDED_WIKI_ITEMS: &str = include_str!("../wiki_items_seed.json");
const EMBEDDED_CRAFT_RECIPES: &str = include_str!("../wiki_crafts_seed.json");
const EMBEDDED_RESOURCE_NAMES: &str = include_str!("../resource_names_seed.json");

fn parse_embedded_json<T: serde::de::DeserializeOwned + Default>(json: &str) -> T {
    serde_json::from_str(json).unwrap_or_default()
}

pub(super) fn embedded_wiki_items() -> Vec<ScrapedItem> {
    let items = parse_embedded_json(EMBEDDED_WIKI_ITEMS);
    merge::finalize_scraped_items(items)
}

pub(super) fn embedded_resource_names() -> Vec<String> {
    parse_embedded_json(EMBEDDED_RESOURCE_NAMES)
}

pub(super) fn embedded_craft_recipes() -> Vec<ScrapedCraftRecipe> {
    parse_embedded_json(EMBEDDED_CRAFT_RECIPES)
}
