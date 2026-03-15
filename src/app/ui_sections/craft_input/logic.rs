use strsim::jaro_winkler;
use crate::app::MdcraftApp;
use crate::app::npc_price_rules::fixed_npc_price_input;
use crate::parse::parse_price_flag;


pub(super) fn lookup_cached_npc_price_input(app: &MdcraftApp, item_name: &str) -> Option<String> {
    let normalized = item_name.trim().to_lowercase();
    // Tenta casar exatamente
    if let Some(price) = app.wiki_cached_items
        .iter()
        .find(|entry| entry.name.trim().to_lowercase() == normalized)
        .and_then(|entry| entry.npc_price.clone()) {
        return Some(price);
    }

    // Tenta casar removendo 's' do final (plural simples)
    if normalized.ends_with('s') {
        let singular = normalized.trim_end_matches('s');
        if let Some(price) = app.wiki_cached_items
            .iter()
            .find(|entry| entry.name.trim().to_lowercase() == singular)
            .and_then(|entry| entry.npc_price.clone()) {
            return Some(price);
        }
    }
    // Tenta casar adicionando 's' (caso o JSON esteja no plural)
    let plural = format!("{}s", normalized);
    if let Some(price) = app.wiki_cached_items
        .iter()
        .find(|entry| entry.name.trim().to_lowercase() == plural)
        .and_then(|entry| entry.npc_price.clone()) {
        return Some(price);
    }

    // Fuzzy matching: encontra o nome mais parecido usando Jaro-Winkler
    let (_best_name, best_score, best_price) = app.wiki_cached_items.iter()
        .filter_map(|entry| {
            let entry_name = entry.name.trim().to_lowercase();
            let score = jaro_winkler(&normalized, &entry_name);
            entry.npc_price.as_ref().map(|price| (entry_name, score, price.clone()))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or((String::new(), 0.0, String::new()));
    if best_score > 0.90 {
        return Some(best_price);
    }

    // Busca tolerante a plural/singular no JSON de preços fixos
    if let Some(price) = fixed_npc_price_input(item_name) {
        return Some(price);
    }
    // Tenta singular
    let normalized = item_name.trim().to_lowercase();
    if normalized.ends_with('s') {
        let singular = normalized.trim_end_matches('s');
        if let Some(price) = fixed_npc_price_input(singular) {
            return Some(price);
        }
    }
    // Tenta plural
    let plural = format!("{}s", normalized);
    if let Some(price) = fixed_npc_price_input(&plural) {
        return Some(price);
    }
    None
}

pub fn apply_cached_npc_price_if_available(app: &MdcraftApp, item: &mut crate::model::Item) {
    let Some(npc_input) = lookup_cached_npc_price_input(app, &item.nome) else {
        return;
    };

    if parse_price_flag(&npc_input).is_err() {
        return;
    }

    item.preco_input = npc_input;
    item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0.0);
    item.valor_total = item.preco_unitario * item.quantidade as f64;
}

