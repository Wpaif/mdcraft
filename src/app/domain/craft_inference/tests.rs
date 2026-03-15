use super::{build_craft_recipe_name_index, infer_craft_name_from_items};
use super::fuzzy::fuzzy_resolve_ingredient;
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
            quantidade_base: 35,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: false,
            preco_input: String::new(),
        },
        Item {
            nome: "Metal Scrap".to_string(),
            quantidade: 500,
            quantidade_base: 500,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: true,
            preco_input: String::new(),
        },
    ];

    let result = infer_craft_name_from_items(&items, &recipes, &index);
    assert_eq!(result.as_deref(), Some("Drone"));
}
