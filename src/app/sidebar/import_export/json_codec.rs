use serde::{Deserialize, Serialize};

use crate::app::SavedCraft;

const MAX_IMPORT_JSON_BYTES: usize = 2_000_000;
const MAX_IMPORTED_CRAFTS: usize = 1_000;
const MAX_NAME_CHARS: usize = 120;
const MAX_RECIPE_TEXT_CHARS: usize = 20_000;
const MAX_PRICE_INPUT_CHARS: usize = 32;
const MAX_ITEM_PRICES_PER_CRAFT: usize = 500;
const MAX_ITEM_NAME_CHARS: usize = 120;
const MAX_ITEM_PRICE_INPUT_CHARS: usize = 32;

#[derive(Serialize)]
struct ExportPayload<'a> {
    saved_crafts: &'a [SavedCraft],
}

pub(super) fn build_export_json(saved_crafts: &[SavedCraft]) -> Result<String, String> {
    let payload = ExportPayload { saved_crafts };
    serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Erro ao gerar JSON de exportação: {err}"))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImportPayload {
    List(Vec<SavedCraft>),
    SavedCrafts { saved_crafts: Vec<SavedCraft> },
    Recipes { recipes: Vec<SavedCraft> },
}

pub(super) fn parse_imported_saved_crafts(raw_json: &str) -> Result<Vec<SavedCraft>, String> {
    if raw_json.len() > MAX_IMPORT_JSON_BYTES {
        return Err("JSON muito grande para importacao.".to_string());
    }

    let payload: ImportPayload = serde_json::from_str(raw_json)
        .map_err(|err| format!("JSON inválido para importação: {err}"))?;

    let mut crafts = match payload {
        ImportPayload::List(items) => items,
        ImportPayload::SavedCrafts { saved_crafts } => saved_crafts,
        ImportPayload::Recipes { recipes } => recipes,
    };

    if crafts.len() > MAX_IMPORTED_CRAFTS {
        return Err("JSON possui receitas demais para importacao.".to_string());
    }

    sanitize_imported_crafts(&mut crafts)?;

    Ok(crafts)
}

fn sanitize_imported_crafts(crafts: &mut [SavedCraft]) -> Result<(), String> {
    for craft in crafts {
        if craft.name.chars().count() > MAX_NAME_CHARS {
            return Err("Nome de receita excede o limite permitido.".to_string());
        }
        if craft.recipe_text.chars().count() > MAX_RECIPE_TEXT_CHARS {
            return Err("Texto da receita excede o limite permitido.".to_string());
        }
        if craft.sell_price_input.chars().count() > MAX_PRICE_INPUT_CHARS {
            return Err("Preco final excede o limite permitido.".to_string());
        }
        if craft.item_prices.len() > MAX_ITEM_PRICES_PER_CRAFT {
            return Err("Quantidade de precos por receita excede o limite permitido.".to_string());
        }

        craft.name = sanitize_single_line(&craft.name);
        craft.recipe_text = sanitize_multiline(&craft.recipe_text);
        craft.sell_price_input = sanitize_single_line(&craft.sell_price_input);

        for item_price in &mut craft.item_prices {
            if item_price.item_name.chars().count() > MAX_ITEM_NAME_CHARS {
                return Err("Nome de item excede o limite permitido.".to_string());
            }
            if item_price.price_input.chars().count() > MAX_ITEM_PRICE_INPUT_CHARS {
                return Err("Preco de item excede o limite permitido.".to_string());
            }

            item_price.item_name = sanitize_single_line(&item_price.item_name);
            item_price.price_input = sanitize_single_line(&item_price.price_input);
        }
    }

    Ok(())
}

fn sanitize_single_line(raw: &str) -> String {
    raw.chars()
        .filter(|&c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

fn sanitize_multiline(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c == '\n' || c == '\t' || !c.is_control() {
            out.push(c);
        }
    }

    out.trim().to_string()
}

pub(super) fn format_json_pretty(raw_json: &str) -> Result<String, String> {
    let value = serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| format!("JSON inválido para formatação: {err}"))?;

    serde_json::to_string_pretty(&value).map_err(|err| format!("Erro ao formatar JSON: {err}"))
}
