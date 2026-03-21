pub(crate) fn output_multiplier_from_craft_name(name: &str) -> u64 {
    let name = name.trim();
    let Some(paren_start) = name.rfind('(') else {
        return 1;
    };
    let Some(paren_end_rel) = name[paren_start..].find(')') else {
        return 1;
    };

    let inside = name[paren_start + 1..paren_start + paren_end_rel].trim();
    if inside.is_empty() {
        return 1;
    }

    let digits: String = inside.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return 1;
    }

    let rest = inside[digits.len()..].trim().to_ascii_lowercase();
    if !rest.starts_with('x') {
        return 1;
    }

    digits
        .parse::<u64>()
        .ok()
        .filter(|&v| v >= 1 && v <= 10_000)
        .unwrap_or(1)
}

pub(crate) fn should_show_per_item_toggle(output_multiplier_per_craft: u64) -> bool {
    compute_total_output(output_multiplier_per_craft, 1) > 1
}

pub(crate) fn should_show_per_item_toggle_for_craft_count(
    output_multiplier_per_craft: u64,
    craft_count: u64,
) -> bool {
    compute_total_output(output_multiplier_per_craft, craft_count) > 1
}

pub(crate) fn compute_total_output(output_multiplier_per_craft: u64, craft_count: u64) -> u64 {
    let per_craft = output_multiplier_per_craft.max(1);
    per_craft.saturating_mul(craft_count.max(1))
}

pub(crate) fn compute_total_revenue(
    input_value: f64,
    sell_price_is_per_item: bool,
    output_multiplier_per_craft: u64,
    craft_count: u64,
) -> f64 {
    if input_value <= 0.0 {
        return 0.0;
    }
    if !sell_price_is_per_item {
        return input_value;
    }

    let total_output = compute_total_output(output_multiplier_per_craft, craft_count);
    input_value * total_output as f64
}

#[cfg(test)]
mod tests {
    use super::{
        compute_total_output, compute_total_revenue, output_multiplier_from_craft_name,
        should_show_per_item_toggle, should_show_per_item_toggle_for_craft_count,
    };

    #[test]
    fn output_multiplier_from_craft_name_parses_x_suffix() {
        assert_eq!(output_multiplier_from_craft_name("Beast Ball (15x)"), 15);
        assert_eq!(output_multiplier_from_craft_name("Poke Ball (100x)"), 100);
        assert_eq!(output_multiplier_from_craft_name("Dresser (2x)"), 2);
    }

    #[test]
    fn output_multiplier_from_craft_name_defaults_to_1_on_invalid() {
        assert_eq!(output_multiplier_from_craft_name("Iron Ore"), 1);
        assert_eq!(output_multiplier_from_craft_name("Foo (x)"), 1);
        assert_eq!(output_multiplier_from_craft_name("Foo (15)"), 1);
        assert_eq!(output_multiplier_from_craft_name("Foo (0x)"), 1);
        assert_eq!(output_multiplier_from_craft_name("Foo (999999x)"), 1);
        assert_eq!(output_multiplier_from_craft_name("Foo (15X)"), 15);
    }

    #[test]
    fn should_show_per_item_toggle_only_for_multi_output() {
        assert!(!should_show_per_item_toggle(1));
        assert!(should_show_per_item_toggle(2));
    }

    #[test]
    fn should_show_per_item_toggle_for_craft_count_shows_when_quantity_gt_1() {
        assert!(!should_show_per_item_toggle_for_craft_count(1, 1));
        assert!(should_show_per_item_toggle_for_craft_count(1, 2));
        assert!(should_show_per_item_toggle_for_craft_count(2, 1));
    }

    #[test]
    fn compute_total_output_multiplies_by_craft_count() {
        assert_eq!(compute_total_output(15, 1), 15);
        assert_eq!(compute_total_output(15, 2), 30);
        assert_eq!(compute_total_output(1, 2), 2);
    }

    #[test]
    fn compute_total_revenue_respects_per_item_flag() {
        assert_eq!(compute_total_revenue(10.0, false, 15, 2), 10.0);
        assert_eq!(compute_total_revenue(10.0, true, 15, 2), 300.0);
        assert_eq!(compute_total_revenue(0.0, true, 15, 2), 0.0);
    }
}
