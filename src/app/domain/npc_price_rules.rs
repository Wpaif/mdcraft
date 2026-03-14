//! Domain rules for NPC price overrides not provided (or not stable) in wiki data.

const FIXED_NPC_PRICE_OVERRIDES: [(&str, &str); 2] = [
    ("compressed nightmare gems", "25k"),
    ("neutral essence", "1k"),
];

pub(crate) fn fixed_npc_price_input(item_name: &str) -> Option<&'static str> {
    let normalized = item_name.trim().to_lowercase();
    FIXED_NPC_PRICE_OVERRIDES
        .iter()
        .find_map(|(name, price)| (*name == normalized).then_some(*price))
}

pub(crate) fn fixed_npc_price_entries() -> &'static [(&'static str, &'static str)] {
    &FIXED_NPC_PRICE_OVERRIDES
}

#[cfg(test)]
mod tests {
    use super::{fixed_npc_price_entries, fixed_npc_price_input};

    #[test]
    fn fixed_npc_price_input_matches_case_and_spacing_insensitively() {
        assert_eq!(
            fixed_npc_price_input("  Compressed Nightmare Gems  "),
            Some("25k")
        );
        assert_eq!(fixed_npc_price_input("Neutral Essence"), Some("1k"));
    }

    #[test]
    fn fixed_npc_price_entries_exposes_all_known_overrides() {
        let entries = fixed_npc_price_entries();
        assert!(entries.contains(&("compressed nightmare gems", "25k")));
        assert!(entries.contains(&("neutral essence", "1k")));
    }
}
