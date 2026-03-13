pub fn format_game_units(valor: f64) -> String {
    if valor >= 1_000_000.0 {
        format!("{:.1}KK", valor / 1_000_000.0)
    } else if valor >= 1_000.0 {
        let k = valor / 1_000.0;

        if (k * 10.0) % 10.0 == 0.0 {
            format!("{}k", k as u64)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        if valor.fract().abs() < f64::EPSILON {
            format!("{:.0}", valor)
        } else {
            let mut s = format!("{:.2}", valor);
            while s.contains('.') && s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_game_units;

    #[test]
    fn format_game_units_uses_kk_for_millions() {
        assert_eq!(format_game_units(1_500_000.0), "1.5KK");
    }

    #[test]
    fn format_game_units_uses_integer_k_when_exact_thousands() {
        assert_eq!(format_game_units(3_000.0), "3k");
    }

    #[test]
    fn format_game_units_uses_decimal_k_when_needed() {
        assert_eq!(format_game_units(2_500.0), "2.5k");
    }

    #[test]
    fn format_game_units_formats_small_integers_without_decimals() {
        assert_eq!(format_game_units(42.0), "42");
    }

    #[test]
    fn format_game_units_trims_trailing_zeros_for_small_decimals() {
        assert_eq!(format_game_units(12.30), "12.3");
        assert_eq!(format_game_units(12.05), "12.05");
    }
}
