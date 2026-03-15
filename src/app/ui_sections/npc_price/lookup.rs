use std::collections::HashMap;

use crate::app::MdcraftApp;
use crate::app::npc_price_rules::fixed_npc_price_entries;
use crate::parse::parse_price_flag;

use super::NpcPriceComparison;

pub(super) fn build_npc_price_lookup(app: &MdcraftApp) -> HashMap<String, f64> {
    let mut lookup = HashMap::new();

    for entry in &app.wiki_cached_items {
        let Some(raw_price) = &entry.npc_price else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(raw_price) else {
            continue;
        };

        lookup.insert(entry.name.trim().to_lowercase(), parsed);
    }

    for (fixed_name, raw_price) in fixed_npc_price_entries() {
        if let Ok(parsed) = parse_price_flag(&raw_price) {
            lookup.insert(fixed_name.trim().to_lowercase(), parsed);
        }
    }

    lookup
}

pub(super) fn compare_item_price_with_npc(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<NpcPriceComparison> {
    let entered = parse_price_flag(&item.preco_input).ok()?;
    // Use the same logic as npc_price_for_item
    let npc_price = {
        use strsim::jaro_winkler;
        let normalized = item.nome.trim().to_lowercase();
        // 1. Exato
        if let Some(val) = npc_lookup.get(&normalized) {
            Some(*val)
        } else if normalized.ends_with('s') {
            let singular = normalized.trim_end_matches('s');
            npc_lookup.get(singular).copied()
        } else {
            let plural = format!("{}s", normalized);
            if let Some(val) = npc_lookup.get(&plural) {
                Some(*val)
            } else {
                // Fuzzy
                let (_best_name, best_score, best_val) = npc_lookup.iter()
                    .map(|(name, val)| (name, jaro_winkler(&normalized, name), val))
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap_or((&String::new(), 0.0, &0.0));
                if best_score > 0.90 {
                    Some(*best_val)
                } else {
                    None
                }
            }
        }
    }?;

    let eps = 1e-9;
    if (entered - npc_price).abs() < eps {
        Some(NpcPriceComparison::Equal)
    } else if entered > npc_price {
        Some(NpcPriceComparison::HigherThanNpc)
    } else {
        Some(NpcPriceComparison::LowerThanNpc)
    }
}

pub(super) fn npc_price_for_item(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<f64> {
    use strsim::jaro_winkler;
    let normalized = item.nome.trim().to_lowercase();
    // 1. Exato
    if let Some(val) = npc_lookup.get(&normalized) {
        return Some(*val);
    }
    // 2. Singular
    if normalized.ends_with('s') {
        let singular = normalized.trim_end_matches('s');
        if let Some(val) = npc_lookup.get(singular) {
            return Some(*val);
        }
    }
    // 3. Plural
    let plural = format!("{}s", normalized);
    if let Some(val) = npc_lookup.get(&plural) {
        return Some(*val);
    }
    // 4. Fuzzy
    let (_best_name, best_score, best_val) = npc_lookup.iter()
        .map(|(name, val)| (name, jaro_winkler(&normalized, name), val))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or((&String::new(), 0.0, &0.0));
    if best_score > 0.90 {
        return Some(*best_val);
    }
    None
}

pub(super) fn should_show_npc_price_icon(item_name: &str) -> bool {
    !item_name.trim().eq_ignore_ascii_case("diamond")
}
