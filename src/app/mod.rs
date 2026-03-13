//! Top-level application logic and state.
//!
//! The previous `src/app.rs` file has been broken into smaller submodules
//! following a directory-based layout.  Each feature of the UI has its own
//! file:
//!
//! * `theme_state` – theme enum and toggle button logic.
//! * `styles` – custom egui styling and emoji font support.
//! * `price` – price status indicator helpers.
//! * `sidebar` – docked collapsible left menu.
//! * `ui` – the implementation of `eframe::App` and the main view.

use crate::data::wiki_scraper::{
    ScrapeRefreshData, ScrapedCraftRecipe, ScrapedItem, embedded_craft_recipes,
    embedded_resource_names, embedded_wiki_items,
};
use crate::model::Item;
use crate::parse::parse_price_flag;
use dark_light::Mode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Instant;

// `Item` is needed for the application state. Parser functions and formatter are
// now imported in `ui.rs` where the UI logic lives.

// re-export the Theme type so callers can refer to `app::Theme`.
pub use theme_state::Theme;

const APP_SETTINGS_KEY: &str = "mdcraft.app_settings";

mod price;
mod sidebar;
mod sqlite;
mod styles;
mod theme_state;
mod ui;
mod ui_sections;

pub(super) fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        Ok(Mode::Dark) => Theme::Dark,
        Ok(Mode::Light) | Ok(Mode::Unspecified) | Err(_) => Theme::Light,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedItemPrice {
    pub item_name: String,
    pub price_input: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedCraft {
    pub name: String,
    pub recipe_text: String,
    pub sell_price_input: String,
    #[serde(default)]
    pub item_prices: Vec<SavedItemPrice>,
}

const FIXED_NPC_PRICE_COMPRESSED_NIGHTMARE_GEMS: &str = "25k";
const FIXED_NPC_PRICE_NEUTRAL_ESSENCE: &str = "1k";

pub fn fixed_npc_price_input(item_name: &str) -> Option<&'static str> {
    let normalized = item_name.trim().to_lowercase();
    match normalized.as_str() {
        "compressed nightmare gems" => Some(FIXED_NPC_PRICE_COMPRESSED_NIGHTMARE_GEMS),
        "neutral essence" => Some(FIXED_NPC_PRICE_NEUTRAL_ESSENCE),
        _ => None,
    }
}

fn normalized_item_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub(crate) fn normalized_ingredient_key(name: &str) -> String {
    let key = name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase();

    // Normalize plural/singular so "Nightmare Gem" and "Nightmare Gems" map to
    // the same key regardless of which form the user typed.
    key.strip_suffix('s').map(str::to_owned).unwrap_or(key)
}

/// Retorna a lista de nomes canônicos únicos de todos os ingredientes das receitas.
fn build_ingredient_vocabulary(recipes: &[ScrapedCraftRecipe]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut vocab = Vec::new();
    for recipe in recipes {
        for ingredient in &recipe.ingredients {
            if seen.insert(ingredient.name.clone()) {
                vocab.push(ingredient.name.clone());
            }
        }
    }
    vocab
}

/// Tenta resolver `raw` para o nome canônico mais próximo em `vocab`.
///
/// Lógica:
/// 1. Match exato (após normalização) — sem custo.
/// 2. Fallback fuzzy via similaridade de Levenshtein normalizada.
///    Exige score ≥ 0.80 e que haja exatamente um candidato vencedor.
///    Em caso de empate ou nenhum candidato acima do threshold, retorna `raw`.
fn fuzzy_resolve_ingredient<'a>(raw: &'a str, vocab: &'a [String]) -> &'a str {
    let raw_key = normalized_ingredient_key(raw);

    // 1. Exact match.
    for canonical in vocab {
        if normalized_ingredient_key(canonical) == raw_key {
            return canonical.as_str();
        }
    }

    // 2. Fuzzy match.
    const THRESHOLD: f64 = 0.80;
    let mut best_score = THRESHOLD;
    let mut best_match: Option<&str> = None;
    let mut ambiguous = false;

    for canonical in vocab {
        let key = normalized_ingredient_key(canonical);
        let score = strsim::normalized_levenshtein(&raw_key, &key);

        #[allow(clippy::float_cmp)]
        if score > best_score {
            best_score = score;
            best_match = Some(canonical.as_str());
            ambiguous = false;
        } else if score == best_score && best_match.is_some() {
            ambiguous = true;
        }
    }

    if ambiguous {
        raw
    } else {
        best_match.unwrap_or(raw)
    }
}

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

    if joined.is_empty() { None } else { Some(joined) }
}

fn recipe_quantity_to_u64(quantity: f64) -> Option<u64> {
    let rounded = quantity.round();
    if (quantity - rounded).abs() > 1e-6 || rounded <= 0.0 {
        return None;
    }

    Some(rounded as u64)
}

fn ingredient_quantities_from_items(items: &[Item]) -> HashMap<String, u64> {
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

fn ingredient_quantities_from_recipe(recipe: &ScrapedCraftRecipe) -> Option<HashMap<String, u64>> {
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

fn is_recipe_multiple_of_items(
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

pub(crate) fn craft_signature_from_items(items: &[Item]) -> Option<String> {
    compose_craft_signature(ingredient_quantities_from_items(items).into_iter().collect())
}

pub(crate) fn craft_signature_from_recipe(recipe: &ScrapedCraftRecipe) -> Option<String> {
    compose_craft_signature(ingredient_quantities_from_recipe(recipe)?.into_iter().collect())
}

pub(crate) fn infer_craft_name_from_items(
    items: &[Item],
    recipes: &[ScrapedCraftRecipe],
    recipe_name_by_signature: &HashMap<String, String>,
) -> Option<String> {
    // 1. Tenta match exato pela assinatura já indexada.
    let item_signature = craft_signature_from_items(items)?;
    if let Some(exact_name) = recipe_name_by_signature.get(&item_signature) {
        return Some(exact_name.clone());
    }

    // 2. Constrói vocabulário canônico e resolve nomes fuzzy.
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

    // 2a. Tenta assinatura exata com nomes resolvidos.
    if let Some(resolved_sig) = craft_signature_from_items(&resolved) {
        if resolved_sig != item_signature {
            if let Some(name) = recipe_name_by_signature.get(&resolved_sig) {
                return Some(name.clone());
            }
        }
    }

    // 2b. Fallback: verifica múltiplo de receita com nomes resolvidos.
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

pub(crate) fn build_craft_recipe_name_index(
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

pub fn capture_saved_item_prices(items: &[Item]) -> Vec<SavedItemPrice> {
    let mut result: Vec<SavedItemPrice> = items
        .iter()
        .filter_map(|item| {
            let price_input = item.preco_input.trim();
            if price_input.is_empty() {
                return None;
            }

            Some(SavedItemPrice {
                item_name: item.nome.clone(),
                price_input: price_input.to_string(),
            })
        })
        .collect();

    result.sort_by(|a, b| a.item_name.cmp(&b.item_name));
    result
}

pub fn apply_saved_item_prices(items: &mut [Item], saved_prices: &[SavedItemPrice]) {
    let lookup: HashMap<String, &str> = saved_prices
        .iter()
        .map(|saved| {
            (
                normalized_item_key(&saved.item_name),
                saved.price_input.as_str(),
            )
        })
        .collect();

    for item in items {
        let key = normalized_item_key(&item.nome);
        let Some(saved_input) = lookup.get(&key).copied() else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(saved_input) else {
            continue;
        };

        item.preco_input = saved_input.to_string();
        item.preco_unitario = parsed;
        item.valor_total = parsed * item.quantidade as f64;
    }
}

/// The application state that is passed to `eframe`.
///
/// In GKT4 terms, this is the *model* for the main window; the view logic lives
/// in `ui.rs` and helpers are in other submodules.
pub struct MdcraftApp {
    pub input_text: String,
    pub items: Vec<Item>,
    pub sell_price_input: String,
    pub resource_list: Vec<String>,
    pub fonts_loaded: bool,
    pub theme: Theme,
    pub follow_system_theme: bool,
    pub sidebar_open: bool,
    pub saved_crafts: Vec<SavedCraft>,
    pub pending_craft_name: String,
    pub awaiting_craft_name: bool,
    pub focus_craft_name_input: bool,
    pub pending_delete_index: Option<usize>,
    pub active_saved_craft_index: Option<usize>,
    pub awaiting_import_json: bool,
    pub import_json_input: String,
    pub import_feedback: Option<String>,
    pub awaiting_export_json: bool,
    pub export_json_output: String,
    pub export_feedback: Option<String>,
    pub wiki_sync_feedback: Option<String>,
    pub wiki_cached_items: Vec<ScrapedItem>,
    pub craft_recipes_cache: Vec<ScrapedCraftRecipe>,
    pub craft_recipe_name_by_signature: HashMap<String, String>,
    pub wiki_http_etag_cache: HashMap<String, String>,
    pub wiki_http_last_modified_cache: HashMap<String, String>,
    pub wiki_refresh_in_progress: bool,
    pub wiki_refresh_rx: Option<Receiver<Result<ScrapeRefreshData, String>>>,
    pub wiki_sync_success_anim_started_at: Option<Instant>,
    pub wiki_refresh_started_on_launch: bool,
    pub wiki_last_sync_unix_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    theme: Option<Theme>,
    follow_system_theme: Option<bool>,
    #[serde(default)]
    saved_crafts: Vec<SavedCraft>,
    #[serde(default)]
    wiki_cached_items: Vec<ScrapedItem>,
    #[serde(default)]
    wiki_http_etag_cache: HashMap<String, String>,
    #[serde(default)]
    wiki_http_last_modified_cache: HashMap<String, String>,
    wiki_last_sync_unix_seconds: Option<u64>,
}

impl MdcraftApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        if let Some(storage) = cc.storage
            && let Some(raw) = storage.get_string(APP_SETTINGS_KEY)
            && let Ok(settings) = serde_json::from_str::<AppSettings>(&raw)
        {
            if let Some(theme) = settings.theme {
                app.theme = theme;
            }
            if let Some(follow_system_theme) = settings.follow_system_theme {
                app.follow_system_theme = follow_system_theme;
            }
            app.saved_crafts = settings.saved_crafts;
            if !settings.wiki_cached_items.is_empty() {
                app.wiki_cached_items = settings.wiki_cached_items;
            }
            app.wiki_http_etag_cache = settings.wiki_http_etag_cache;
            app.wiki_http_last_modified_cache = settings.wiki_http_last_modified_cache;
            app.wiki_last_sync_unix_seconds = settings.wiki_last_sync_unix_seconds;
        }

        // Keep tests deterministic by avoiding local SQLite state leakage.
        if !cfg!(test) {
            if let Ok(saved_crafts) = sqlite::load_saved_crafts_from_sqlite() {
                app.saved_crafts = saved_crafts;
            }
        }

        app
    }

    pub(crate) fn persist_saved_crafts_to_sqlite(&self) {
        if let Err(err) = sqlite::save_saved_crafts_to_sqlite(&self.saved_crafts) {
            eprintln!("Falha ao persistir receitas no SQLite: {err}");
        }
    }

    pub fn save_app_settings(&self, storage: &mut dyn eframe::Storage) {
        let settings = AppSettings {
            theme: Some(self.theme),
            follow_system_theme: Some(self.follow_system_theme),
            saved_crafts: self.saved_crafts.clone(),
            wiki_cached_items: self.wiki_cached_items.clone(),
            wiki_http_etag_cache: self.wiki_http_etag_cache.clone(),
            wiki_http_last_modified_cache: self.wiki_http_last_modified_cache.clone(),
            wiki_last_sync_unix_seconds: self.wiki_last_sync_unix_seconds,
        };

        if let Ok(raw) = serde_json::to_string(&settings) {
            storage.set_string(APP_SETTINGS_KEY, raw);
        }

        self.persist_saved_crafts_to_sqlite();
    }

    pub(crate) fn rebuild_craft_recipe_name_index(&mut self) {
        self.craft_recipe_name_by_signature = build_craft_recipe_name_index(&self.craft_recipes_cache);
    }
}

impl Default for MdcraftApp {
    fn default() -> Self {
        let system_theme = detect_system_theme();
        let wiki_cached_items = embedded_wiki_items();
        let craft_recipes_cache = embedded_craft_recipes();
        let craft_recipe_name_by_signature = build_craft_recipe_name_index(&craft_recipes_cache);
        let resource_list = embedded_resource_names();

        Self {
            input_text: String::new(),
            items: Vec::new(),
            sell_price_input: String::new(),
            resource_list,
            fonts_loaded: false,
            theme: system_theme,
            follow_system_theme: true,
            sidebar_open: true,
            saved_crafts: Vec::new(),
            pending_craft_name: String::new(),
            awaiting_craft_name: false,
            focus_craft_name_input: false,
            pending_delete_index: None,
            active_saved_craft_index: None,
            awaiting_import_json: false,
            import_json_input: String::new(),
            import_feedback: None,
            awaiting_export_json: false,
            export_json_output: String::new(),
            export_feedback: None,
            wiki_sync_feedback: None,
            wiki_cached_items,
            craft_recipes_cache,
            craft_recipe_name_by_signature,
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_refresh_in_progress: false,
            wiki_refresh_rx: None,
            wiki_sync_success_anim_started_at: None,
            wiki_refresh_started_on_launch: false,
            wiki_last_sync_unix_seconds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::{CreationContext, Storage, egui};

    use super::{
        APP_SETTINGS_KEY, AppSettings, MdcraftApp, SavedCraft, Theme, detect_system_theme,
    };
    use crate::data::wiki_scraper::{ScrapedItem, WikiSource};

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_string(), value);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn default_app_starts_with_expected_flags() {
        let app = MdcraftApp::default();
        assert!(app.follow_system_theme);
        assert!(app.sidebar_open);
        assert!(!app.fonts_loaded);
        assert!(app.items.is_empty());
    }

    #[test]
    fn save_app_settings_writes_json_payload() {
        let mut app = MdcraftApp::default();
        app.theme = Theme::Dark;
        app.follow_system_theme = false;
        app.saved_crafts.push(SavedCraft {
            name: "Receita A".to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "10k".to_string(),
            item_prices: vec![],
        });

        let mut storage = MemoryStorage::default();
        app.save_app_settings(&mut storage);

        let raw = storage
            .get_string(APP_SETTINGS_KEY)
            .expect("settings JSON should be stored");
        assert!(raw.contains("Receita A"));

        storage.flush();

        storage.flush();
    }

    #[test]
    fn from_creation_context_restores_saved_settings() {
        let settings = AppSettings {
            theme: Some(Theme::Dark),
            follow_system_theme: Some(false),
            saved_crafts: vec![SavedCraft {
                name: "Restaurada".to_string(),
                recipe_text: "2 Screw".to_string(),
                sell_price_input: "4k".to_string(),
                item_prices: vec![],
            }],
            wiki_cached_items: vec![],
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_last_sync_unix_seconds: None,
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.theme, Theme::Dark);
        assert!(!app.follow_system_theme);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.saved_crafts[0].name, "Restaurada");
    }

    #[test]
    fn detect_system_theme_returns_valid_variant() {
        let theme = detect_system_theme();
        assert!(matches!(theme, Theme::Light | Theme::Dark));
    }

    #[test]
    fn from_creation_context_ignores_invalid_settings_json() {
        let mut storage = MemoryStorage::default();
        storage.set_string(APP_SETTINGS_KEY, "{invalid-json".to_string());

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert!(app.saved_crafts.is_empty());
    }

    #[test]
    fn from_creation_context_applies_partial_settings() {
        let settings = AppSettings {
            theme: Some(Theme::Dark),
            follow_system_theme: None,
            saved_crafts: vec![],
            wiki_cached_items: vec![],
            wiki_http_etag_cache: HashMap::new(),
            wiki_http_last_modified_cache: HashMap::new(),
            wiki_last_sync_unix_seconds: None,
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.theme, Theme::Dark);
        assert!(app.follow_system_theme);
    }

    #[test]
    fn from_creation_context_restores_wiki_cache_and_keeps_resource_seed() {
        let settings = AppSettings {
            theme: None,
            follow_system_theme: None,
            saved_crafts: vec![],
            wiki_cached_items: vec![
                ScrapedItem {
                    name: "Ancient Wire".to_string(),
                    npc_price: Some("12k".to_string()),
                    sources: vec![WikiSource::Loot],
                },
                ScrapedItem {
                    name: "Gear Nose".to_string(),
                    npc_price: None,
                    sources: vec![WikiSource::Nightmare],
                },
            ],
            wiki_http_etag_cache: HashMap::from([(
                String::from("https://wiki"),
                String::from("etag1"),
            )]),
            wiki_http_last_modified_cache: HashMap::from([(
                String::from("https://wiki"),
                String::from("Wed, 21 Oct 2015 07:28:00 GMT"),
            )]),
            wiki_last_sync_unix_seconds: Some(1_700_000_000),
        };

        let mut storage = MemoryStorage::default();
        storage.set_string(
            APP_SETTINGS_KEY,
            serde_json::to_string(&settings).expect("settings should serialize"),
        );

        let mut cc = CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);

        let app = MdcraftApp::from_creation_context(&cc);
        assert_eq!(app.wiki_cached_items.len(), 2);
        assert!(app.resource_list.contains(&"tech data".to_string()));
        assert!(!app.resource_list.contains(&"ancient wire".to_string()));
        assert_eq!(
            app.wiki_http_etag_cache
                .get("https://wiki")
                .map(String::as_str),
            Some("etag1")
        );
        assert_eq!(
            app.wiki_http_last_modified_cache
                .get("https://wiki")
                .map(String::as_str),
            Some("Wed, 21 Oct 2015 07:28:00 GMT")
        );
        assert_eq!(app.wiki_last_sync_unix_seconds, Some(1_700_000_000));
    }

    #[test]
    fn fuzzy_resolve_returns_exact_match_on_same_name() {
        let vocab = vec!["Brutal Fins".to_string(), "Metal Scraps".to_string()];
        assert_eq!(
            super::fuzzy_resolve_ingredient("Brutal Fins", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_normalises_singular_to_canonical_plural() {
        let vocab = vec!["Brutal Fins".to_string(), "Metal Scraps".to_string()];
        // "Brutal Fin" → key "brutal fin"; "Brutal Fins" → key "brutal fin" — exact key match.
        assert_eq!(
            super::fuzzy_resolve_ingredient("Brutal Fin", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_corrects_single_letter_typo() {
        let vocab = vec!["Brutal Fins".to_string()];
        // "Brutall Fins" tem distância de edição 1 de "Brutal Fins".
        assert_eq!(
            super::fuzzy_resolve_ingredient("Brutall Fins", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_returns_original_when_no_close_match() {
        let vocab = vec!["Metal Scraps".to_string()];
        let raw = "Completely Different Thing";
        assert_eq!(super::fuzzy_resolve_ingredient(raw, &vocab), raw);
    }

    #[test]
    fn infer_craft_name_identifies_after_fuzzy_resolution() {
        use crate::data::wiki_scraper::{CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe};
        use crate::model::Item;

        let recipes = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Engineer,
            rank: CraftRank::S,
            name: "Drone".to_string(),
            ingredients: vec![
                CraftIngredient { name: "Brutal Fins".to_string(), quantity: 35.0 },
                CraftIngredient { name: "Metal Scraps".to_string(), quantity: 500.0 },
            ],
        }];
        let index = super::build_craft_recipe_name_index(&recipes);

        // Usuário digita singular + typo — deve identificar via fuzzy.
        let items = vec![
            Item { nome: "Brutall Fin".to_string(), quantidade: 35, preco_unitario: 0.0, valor_total: 0.0, is_resource: false, preco_input: String::new() },
            Item { nome: "Metal Scrap".to_string(), quantidade: 500, preco_unitario: 0.0, valor_total: 0.0, is_resource: true, preco_input: String::new() },
        ];

        let result = super::infer_craft_name_from_items(&items, &recipes, &index);
        assert_eq!(result.as_deref(), Some("Drone"));
    }
}
