use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WikiSource {
    Loot,
    Nightmare,
    DimensionalZone,
}

pub const ALL_WIKI_SOURCES: [WikiSource; 3] = [
    WikiSource::Loot,
    WikiSource::Nightmare,
    WikiSource::DimensionalZone,
];

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
