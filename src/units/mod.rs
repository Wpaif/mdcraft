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
        format!("{:.0}", valor)
    }
}
