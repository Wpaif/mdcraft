use reqwest::Client as AsyncClient;
pub async fn scrape_all_sources_incremental_async(
    client: &AsyncClient,
    existing: &[ScrapedItem],
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let existing_price_map = merge::build_existing_price_map(existing);
    pipeline::scrape_all_sources_parallel_async(
        client,
        &ALL_WIKI_SOURCES,
        &existing_price_map,
        etag_cache,
        last_modified_cache,
    ).await
}
use std::collections::HashMap;

// ...existing code...

// Visibility conventions:
// - `pub`: scraper API and domain types consumed by app/data layers.
// - `pub(crate)`: cross-module helpers inside this crate.
// - `pub(super)`/private: submodule internals and implementation details.

#[path = "wiki_scraper/crafts/mod.rs"]
pub mod crafts;
#[path = "wiki_scraper/errors.rs"]
mod errors;
#[path = "wiki_scraper/items_parser/mod.rs"]
mod items_parser;
#[path = "wiki_scraper/merge.rs"]
mod merge;
#[path = "wiki_scraper/pipeline.rs"]
mod pipeline;
#[path = "wiki_scraper/seeds.rs"]
mod seeds;
#[path = "wiki_scraper/source_scrape/mod.rs"]
mod source_scrape;
#[path = "wiki_scraper/types.rs"]
mod types;

// ...existing code...
pub use errors::ScrapeError;
pub use types::{
    ALL_CRAFT_PROFESSIONS, ALL_WIKI_SOURCES, CraftIngredient, CraftProfession, CraftRank,
    ScrapeRefreshData, ScrapedCraftRecipe, ScrapedItem, WikiSource,
};

pub fn embedded_wiki_items() -> Vec<ScrapedItem> {
    seeds::embedded_wiki_items()
}

pub fn embedded_resource_names() -> Vec<String> {
    seeds::embedded_resource_names()
}

pub fn embedded_craft_recipes() -> Vec<ScrapedCraftRecipe> {
    seeds::embedded_craft_recipes()
}

pub fn merge_item_lists(existing: &[ScrapedItem], incoming: &[ScrapedItem]) -> Vec<ScrapedItem> {
    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    merge::merge_items(&mut merged, existing.iter().cloned());
    merge::merge_items(&mut merged, incoming.iter().cloned());

    merge::finalize_scraped_items(merged.into_values().collect())
}

pub fn normalized_resource_names(items: &[ScrapedItem]) -> Vec<String> {
    merge::normalized_resource_names(items)
}

// Funções síncronas de scraping removidas após refatoração para async.

#[cfg(test)]
mod tests {
    use super::{
        ScrapedItem, WikiSource, embedded_resource_names, embedded_wiki_items, merge,
        merge_item_lists, normalized_resource_names,
    };
    use std::collections::HashMap;

    #[test]
    fn merge_items_deduplicates_and_merges_sources() {
        let mut merged = HashMap::new();

        merge::merge_items(
            &mut merged,
            vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: None,
                sources: vec![WikiSource::Loot],
            }],
        );

        merge::merge_items(
            &mut merged,
            vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: Some("12k".to_string()),
                sources: vec![WikiSource::Nightmare],
            }],
        );

        let item = merged
            .get("ancient wire")
            .expect("merged item should exist");
        assert_eq!(item.npc_price.as_deref(), Some("12k"));
        assert!(item.sources.contains(&WikiSource::Loot));
        assert!(item.sources.contains(&WikiSource::Nightmare));
    }

    #[test]
    fn normalized_resource_names_sorts_and_deduplicates() {
        let names = normalized_resource_names(&[
            ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: None,
                sources: vec![],
            },
            ScrapedItem {
                name: " ancient wire ".to_string(),
                npc_price: None,
                sources: vec![],
            },
            ScrapedItem {
                name: "Gear Nose".to_string(),
                npc_price: None,
                sources: vec![],
            },
        ]);

        assert_eq!(
            names,
            vec!["ancient wire".to_string(), "gear nose".to_string()]
        );
    }

    #[test]
    fn embedded_wiki_items_loads_seed_data() {
        let items = embedded_wiki_items();
        assert!(!items.is_empty());
        assert!(items.iter().any(|item| item.npc_price.is_some()));
    }

    #[test]
    fn embedded_resource_names_loads_seed_data() {
        let names = embedded_resource_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"tech data".to_string()));
    }

    #[test]
    fn merge_item_lists_keeps_existing_and_updates_prices() {
        let existing = vec![ScrapedItem {
            name: "Tech Data".to_string(),
            npc_price: None,
            sources: vec![WikiSource::Loot],
        }];
        let incoming = vec![ScrapedItem {
            name: "Tech Data".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Nightmare],
        }];

        let merged = merge_item_lists(&existing, &incoming);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].npc_price.as_deref(), Some("1k"));
        assert!(merged[0].sources.contains(&WikiSource::Loot));
        assert!(merged[0].sources.contains(&WikiSource::Nightmare));
    }
}
