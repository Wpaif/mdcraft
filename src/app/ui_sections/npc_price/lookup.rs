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
        if let Ok(parsed) = parse_price_flag(raw_price) {
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
    let npc_price = npc_lookup.get(&item.nome.trim().to_lowercase()).copied()?;

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
    npc_lookup.get(&item.nome.trim().to_lowercase()).copied()
}

pub(super) fn should_show_npc_price_icon(item_name: &str) -> bool {
    !item_name.trim().eq_ignore_ascii_case("diamond")
}
