use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WikiSource {
    Loot,
    Nightmare,
    DimensionalZone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CraftProfession {
    Engineer,
    Professor,
    Stylist,
    Adventurer,
}

impl CraftProfession {
    #[allow(dead_code)]
    pub fn url(self) -> &'static str {
        match self {
            Self::Engineer => {
                "https://wiki.pokexgames.com/index.php?title=Craft_Profiss%C3%B5es_-_Engenheiro&mobileaction=toggle_view_desktop"
            }
            Self::Professor => {
                "https://wiki.pokexgames.com/index.php?title=Craft_Profiss%C3%B5es_-_Professor&mobileaction=toggle_view_desktop"
            }
            Self::Stylist => {
                "https://wiki.pokexgames.com/index.php?title=Craft_Profiss%C3%B5es_-_Estilista&mobileaction=toggle_view_desktop"
            }
            Self::Adventurer => {
                "https://wiki.pokexgames.com/index.php?title=Craft_Profiss%C3%B5es_-_Aventureiro&mobileaction=toggle_view_desktop"
            }
        }
    }
}

#[allow(dead_code)]
pub const ALL_CRAFT_PROFESSIONS: [CraftProfession; 4] = [
    CraftProfession::Engineer,
    CraftProfession::Professor,
    CraftProfession::Stylist,
    CraftProfession::Adventurer,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CraftRank {
    E,
    D,
    C,
    B,
    A,
    S,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CraftIngredient {
    pub name: String,
    pub quantity: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScrapedCraftRecipe {
    pub profession: CraftProfession,
    pub rank: CraftRank,
    pub name: String,
    pub ingredients: Vec<CraftIngredient>,
}

impl WikiSource {
    pub fn url(self) -> &'static str {
        match self {
            Self::Loot => {
                "https://wiki.pokexgames.com/index.php?title=Itens_de_Loot&mobileaction=toggle_view_desktop"
            }
            Self::Nightmare => {
                "https://wiki.pokexgames.com/index.php/Nightmare_Itens#Itens_Comuns-0"
            }
            Self::DimensionalZone => "https://wiki.pokexgames.com/index.php/Dimensional_Zone_Itens",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScrapedItem {
    pub name: String,
    pub npc_price: Option<String>,
    pub sources: Vec<WikiSource>,
}

#[derive(Clone, Debug, Default)]
pub struct ScrapeRefreshData {
    pub items: Vec<ScrapedItem>,
    pub etag_cache: HashMap<String, String>,
    pub last_modified_cache: HashMap<String, String>,
}

fn has_npc_price(item: &ScrapedItem) -> bool {
    item.npc_price
        .as_deref()
        .map(|p| !p.trim().is_empty())
        .unwrap_or(false)
}

pub(super) fn retain_items_with_npc_price(items: Vec<ScrapedItem>) -> Vec<ScrapedItem> {
    items.into_iter().filter(has_npc_price).collect()
}

const EMBEDDED_WIKI_ITEMS: &str = include_str!("wiki_items_seed.json");
const EMBEDDED_CRAFT_RECIPES: &str = include_str!("wiki_crafts_seed.json");
const EMBEDDED_RESOURCE_NAMES: &str = include_str!("resource_names_seed.json");

#[path = "wiki_scraper/crafts.rs"]
mod crafts;
#[path = "wiki_scraper/items_parser.rs"]
mod items_parser;
#[path = "wiki_scraper/source_scrape.rs"]
mod source_scrape;
pub use crafts::CraftScrapeError;

pub fn embedded_wiki_items() -> Vec<ScrapedItem> {
    let items = serde_json::from_str(EMBEDDED_WIKI_ITEMS).unwrap_or_default();
    retain_items_with_npc_price(items)
}

pub fn embedded_resource_names() -> Vec<String> {
    serde_json::from_str(EMBEDDED_RESOURCE_NAMES).unwrap_or_default()
}

pub fn embedded_craft_recipes() -> Vec<ScrapedCraftRecipe> {
    serde_json::from_str(EMBEDDED_CRAFT_RECIPES).unwrap_or_default()
}

pub fn parse_profession_crafts_from_html(
    html: &str,
    profession: CraftProfession,
) -> Vec<ScrapedCraftRecipe> {
    crafts::parse_profession_crafts_from_html(html, profession)
}

#[allow(dead_code)]
pub fn scrape_profession_crafts(
    client: &Client,
    profession: CraftProfession,
) -> Result<Vec<ScrapedCraftRecipe>, CraftScrapeError> {
    crafts::scrape_profession_crafts(client, profession)
}

#[allow(dead_code)]
pub fn scrape_all_profession_crafts(
    client: &Client,
) -> Result<Vec<ScrapedCraftRecipe>, CraftScrapeError> {
    crafts::scrape_all_profession_crafts(client)
}

pub fn merge_item_lists(existing: &[ScrapedItem], incoming: &[ScrapedItem]) -> Vec<ScrapedItem> {
    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    merge_items(&mut merged, existing.to_vec());
    merge_items(&mut merged, incoming.to_vec());

    let mut items: Vec<ScrapedItem> = retain_items_with_npc_price(merged.into_values().collect());
    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

pub fn normalized_resource_names(items: &[ScrapedItem]) -> Vec<String> {
    let mut names: Vec<String> = items
        .iter()
        .map(|item| item.name.trim().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect();

    names.sort();
    names.dedup();
    names
}

#[derive(Debug)]
pub enum ScrapeError {
    Request { source: WikiSource, message: String },
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request { source, message } => {
                write!(f, "failed to fetch {:?} source: {}", source, message)
            }
        }
    }
}

impl std::error::Error for ScrapeError {}

#[allow(dead_code)]
pub fn scrape_all_sources(client: &Client) -> Result<Vec<ScrapedItem>, ScrapeError> {
    scrape_all_sources_incremental(client, &[], &HashMap::new(), &HashMap::new())
        .map(|data| retain_items_with_npc_price(data.items))
}

pub fn scrape_all_sources_incremental(
    client: &Client,
    existing: &[ScrapedItem],
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let sources = [
        WikiSource::Loot,
        WikiSource::Nightmare,
        WikiSource::DimensionalZone,
    ];

    let existing_price_map: HashMap<String, String> = existing
        .iter()
        .filter_map(|item| {
            item.npc_price
                .as_ref()
                .map(|price| (items_parser::normalize_key(&item.name), price.clone()))
        })
        .collect();

    let (tx, rx) = mpsc::channel();
    for source in sources {
        let tx = tx.clone();
        let client = client.clone();
        let existing_price_map = existing_price_map.clone();
        let etag_cache = etag_cache.clone();
        let last_modified_cache = last_modified_cache.clone();

        thread::spawn(move || {
            let result = scrape_source_incremental_with_cache(
                &client,
                source,
                &existing_price_map,
                &etag_cache,
                &last_modified_cache,
            );
            let _ = tx.send(result);
        });
    }
    drop(tx);

    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    let mut merged_etags = etag_cache.clone();
    let mut merged_last_modified = last_modified_cache.clone();
    for _ in 0..sources.len() {
        let source_data = rx.recv().map_err(|err| ScrapeError::Request {
            source: WikiSource::Loot,
            message: format!("parallel scrape channel error: {err}"),
        })??;
        merge_items(&mut merged, source_data.items);
        merged_etags.extend(source_data.etag_cache);
        merged_last_modified.extend(source_data.last_modified_cache);
    }

    let mut items: Vec<ScrapedItem> = merged.into_values().collect();
    items = retain_items_with_npc_price(items);
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ScrapeRefreshData {
        items,
        etag_cache: merged_etags,
        last_modified_cache: merged_last_modified,
    })
}

#[allow(dead_code)]
pub fn scrape_source(client: &Client, source: WikiSource) -> Result<Vec<ScrapedItem>, ScrapeError> {
    scrape_source_incremental_with_cache(
        client,
        source,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    )
    .map(|data| retain_items_with_npc_price(data.items))
}

#[allow(dead_code)]
pub fn scrape_source_incremental(
    client: &Client,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    scrape_source_incremental_with_cache(
        client,
        source,
        existing_price_map,
        &HashMap::new(),
        &HashMap::new(),
    )
}

fn scrape_source_incremental_with_cache(
    client: &Client,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    source_scrape::scrape_source_incremental_with_cache(
        client,
        source,
        existing_price_map,
        etag_cache,
        last_modified_cache,
    )
}

fn merge_items(merged: &mut HashMap<String, ScrapedItem>, new_items: Vec<ScrapedItem>) {
    for item in new_items {
        let key = items_parser::normalize_key(&item.name);

        if let Some(existing) = merged.get_mut(&key) {
            if existing.npc_price.is_none() && item.npc_price.is_some() {
                existing.npc_price = item.npc_price.clone();
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

#[cfg(test)]
mod tests {
    use super::{
        ScrapedItem, WikiSource, embedded_resource_names, embedded_wiki_items, merge_item_lists,
        merge_items, normalized_resource_names,
    };
    use std::collections::HashMap;

    #[test]
    fn merge_items_deduplicates_and_merges_sources() {
        let mut merged = HashMap::new();

        merge_items(
            &mut merged,
            vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: None,
                sources: vec![WikiSource::Loot],
            }],
        );

        merge_items(
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
