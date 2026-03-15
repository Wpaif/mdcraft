//! Domain rules for NPC price overrides not provided (or not stable) in wiki data.


use std::collections::HashMap;
use std::sync::OnceLock;
use std::fs;

static FIXED_NPC_PRICES: OnceLock<HashMap<String, String>> = OnceLock::new();

fn load_fixed_npc_prices() -> HashMap<String, String> {
    let path = "src/data/fixed_npc_prices.json";
    let json = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&json).unwrap_or_default()
}

fn get_fixed_npc_prices() -> &'static HashMap<String, String> {
    FIXED_NPC_PRICES.get_or_init(load_fixed_npc_prices)
}

pub(crate) fn fixed_npc_price_input(item_name: &str) -> Option<String> {
    use strsim::jaro_winkler;
    let normalized = item_name.trim().to_lowercase();
    let prices = get_fixed_npc_prices();

    // 1. Exact match
    if let Some(price) = prices.get(&normalized) {
        return Some(price.clone());
    }

    // 2. Plural/singular tolerant (smarter)
    // Try removing 's', 'es', or adding 's', 'es' as appropriate
    let mut variants = vec![normalized.clone()];
    if normalized.ends_with("es") {
        variants.push(normalized.trim_end_matches("es").to_string());
    }
    if normalized.ends_with('s') {
        variants.push(normalized.trim_end_matches('s').to_string());
    }
    // Try adding 's' and 'es' to the base
    variants.push(format!("{}s", normalized));
    variants.push(format!("{}es", normalized));

    for variant in &variants {
        if let Some(price) = prices.get(variant) {
            return Some(price.clone());
        }
    }

    // 3. Fuzzy match (Jaro-Winkler) for all variants
    let binding = String::new();
    let mut best_score = 0.0;
    let mut best_price = &binding;
    for variant in &variants {
        for (name, price) in prices.iter() {
            let score = jaro_winkler(variant, name);
            if score > best_score {
                best_score = score;
                best_price = price;
            }
        }
    }
    if best_score > 0.90 {
        return Some(best_price.clone());
    }
    None
}

pub(crate) fn fixed_npc_price_entries() -> HashMap<String, String> {
    get_fixed_npc_prices().clone()
}

#[cfg(test)]
mod tests {
    use super::{fixed_npc_price_entries, fixed_npc_price_input};

    #[test]
    fn fixed_npc_price_input_matches_case_and_spacing_insensitively() {
        assert_eq!(
            fixed_npc_price_input("  Compressed Nightmare Gems  "),
            Some("25k".to_string())
        );
        assert_eq!(fixed_npc_price_input("Neutral Essence"), Some("1k".to_string()));
    }

    #[test]
    fn fixed_npc_price_entries_exposes_all_known_overrides() {
        let entries = fixed_npc_price_entries();
        assert_eq!(entries.get("compressed nightmare gems"), Some(&"25k".to_string()));
        assert_eq!(entries.get("neutral essence"), Some(&"1k".to_string()));
    }
}
