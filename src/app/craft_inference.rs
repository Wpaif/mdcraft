//! Craft recipe name inference and ingredient matching.
//!
//! Provides fuzzy resolution of ingredient names, signature-based recipe
//! look-up, and the index builder used by `MdcraftApp`.

use std::collections::HashMap;

use crate::data::wiki_scraper::ScrapedCraftRecipe;
use crate::model::Item;

pub(super) fn normalized_item_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub(crate) fn normalized_ingredient_key(name: &str) -> String {
    let key = name
        .split_whitespace()
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
    compose_craft_signature(
        ingredient_quantities_from_items(items)
            .into_iter()
            .collect(),
    )
}

pub(crate) fn craft_signature_from_recipe(recipe: &ScrapedCraftRecipe) -> Option<String> {
    compose_craft_signature(
        ingredient_quantities_from_recipe(recipe)?
            .into_iter()
            .collect(),
    )
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

#[cfg(test)]
mod tests {
    use super::{
        build_craft_recipe_name_index, fuzzy_resolve_ingredient, infer_craft_name_from_items,
    };
    use crate::data::wiki_scraper::{
        CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe,
    };
    use crate::model::Item;

    #[test]
    fn fuzzy_resolve_returns_exact_match_on_same_name() {
        let vocab = vec!["Brutal Fins".to_string(), "Metal Scraps".to_string()];
        assert_eq!(
            fuzzy_resolve_ingredient("Brutal Fins", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_normalises_singular_to_canonical_plural() {
        let vocab = vec!["Brutal Fins".to_string(), "Metal Scraps".to_string()];
        assert_eq!(
            fuzzy_resolve_ingredient("Brutal Fin", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_corrects_single_letter_typo() {
        let vocab = vec!["Brutal Fins".to_string()];
        assert_eq!(
            fuzzy_resolve_ingredient("Brutall Fins", &vocab),
            "Brutal Fins"
        );
    }

    #[test]
    fn fuzzy_resolve_returns_original_when_no_close_match() {
        let vocab = vec!["Metal Scraps".to_string()];
        let raw = "Completely Different Thing";
        assert_eq!(fuzzy_resolve_ingredient(raw, &vocab), raw);
    }

    #[test]
    fn infer_craft_name_identifies_after_fuzzy_resolution() {
        let recipes = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Engineer,
            rank: CraftRank::S,
            name: "Drone".to_string(),
            ingredients: vec![
                CraftIngredient {
                    name: "Brutal Fins".to_string(),
                    quantity: 35.0,
                },
                CraftIngredient {
                    name: "Metal Scraps".to_string(),
                    quantity: 500.0,
                },
            ],
        }];
        let index = build_craft_recipe_name_index(&recipes);

        let items = vec![
            Item {
                nome: "Brutall Fin".to_string(),
                quantidade: 35,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            Item {
                nome: "Metal Scrap".to_string(),
                quantidade: 500,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: true,
                preco_input: String::new(),
            },
        ];

        let result = infer_craft_name_from_items(&items, &recipes, &index);
        assert_eq!(result.as_deref(), Some("Drone"));
    }
}
