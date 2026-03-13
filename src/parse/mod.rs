use crate::model::Item;

pub fn parse_price_flag(valor: &str) -> Result<f64, String> {
    let valor = valor.trim().to_lowercase().replace(',', ".");

    if valor.is_empty() {
        return Ok(0.0);
    }

    let parsed = if valor.ends_with("kk") {
        let numero = normalize_numeric_literal(valor.trim_end_matches("kk"))
            .parse::<f64>()
            .map_err(|_| "valor inválido")?;

        numero * 1_000_000.0
    } else if valor.ends_with('k') {
        let numero = normalize_numeric_literal(valor.trim_end_matches('k'))
            .parse::<f64>()
            .map_err(|_| "valor inválido")?;

        numero * 1_000.0
    } else {
        normalize_numeric_literal(&valor)
            .parse::<f64>()
            .map_err(|_| "valor inválido".to_string())?
    };

    if !parsed.is_finite() || parsed < 0.0 {
        return Err("valor inválido".to_string());
    }

    Ok(parsed)
}

fn normalize_numeric_literal(raw: &str) -> String {
    let candidate = raw.trim();
    let dot_count = candidate.chars().filter(|&c| c == '.').count();

    if dot_count > 1 {
        return candidate.replace('.', "");
    }

    if let Some((left, right)) = candidate.split_once('.') {
        if !left.is_empty()
            && right.len() == 3
            && left.chars().all(|c| c.is_ascii_digit())
            && right.chars().all(|c| c.is_ascii_digit())
        {
            return format!("{left}{right}");
        }
    }

    candidate.to_string()
}

pub fn parse_clipboard(clipboard_content: &str, resource_list: &[&str]) -> Vec<Item> {
    if clipboard_content.trim().is_empty() {
        return Vec::new();
    }

    let mut items = Vec::new();

    for item_str in clipboard_content.split(',') {
        let item_str = item_str.trim();

        if item_str.is_empty() {
            continue;
        }

        let parts: Vec<&str> = item_str.splitn(2, ' ').collect();

        if parts.len() == 2 {
            if let Ok(quantidade) = parts[0].parse::<u64>() {
                // Remove pontos finais e espaços que podem vir do copy-paste do jogo
                let nome_original = parts[1].trim().trim_end_matches('.');
                let nome_lower = nome_original.to_lowercase();

                // Validação de plural / singular contra a lista de resources base
                let mut is_resource = false;
                for res in resource_list {
                    let res_lower = res.to_lowercase();

                    if nome_lower == res_lower
                        || nome_lower == format!("{}s", res_lower)
                        || nome_lower == format!("{}es", res_lower)
                        || res_lower == format!("{}s", nome_lower)
                        || res_lower == format!("{}es", nome_lower)
                    {
                        is_resource = true;
                        break;
                    }
                }

                items.push(Item {
                    nome: nome_original.to_string(),
                    quantidade,
                    preco_unitario: 0.0,
                    valor_total: 0.0,
                    is_resource,
                    preco_input: String::new(),
                });
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::{parse_clipboard, parse_price_flag};

    fn approx_eq(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "left={left}, right={right}");
    }

    #[test]
    fn parse_price_flag_accepts_empty_as_zero() {
        let parsed = parse_price_flag("").expect("empty price should be valid");
        approx_eq(parsed, 0.0);
    }

    #[test]
    fn parse_price_flag_parses_plain_and_comma_numbers() {
        let plain = parse_price_flag("123.45").expect("plain number should parse");
        let comma = parse_price_flag("123,45").expect("comma decimal should parse");
        let thousand = parse_price_flag("50.000").expect("thousand separator should parse");

        approx_eq(plain, 123.45);
        approx_eq(comma, 123.45);
        approx_eq(thousand, 50_000.0);
    }

    #[test]
    fn parse_price_flag_parses_k_and_kk_suffixes_case_insensitive() {
        let k = parse_price_flag("2k").expect("k suffix should parse");
        let kk = parse_price_flag(" 1.5KK ").expect("kk suffix should parse");

        approx_eq(k, 2_000.0);
        approx_eq(kk, 1_500_000.0);
    }

    #[test]
    fn parse_price_flag_rejects_invalid_negative_and_non_finite_values() {
        assert!(parse_price_flag("abc").is_err());
        assert!(parse_price_flag("-1").is_err());
        assert!(parse_price_flag("inf").is_err());
        assert!(parse_price_flag("1e309").is_err());
    }

    #[test]
    fn parse_price_flag_rejects_invalid_kk_payload() {
        assert!(parse_price_flag("abcKK").is_err());
    }

    #[test]
    fn parse_clipboard_returns_empty_for_blank_input() {
        let items = parse_clipboard("   ", &["wood"]);
        assert!(items.is_empty());
    }

    #[test]
    fn parse_clipboard_parses_valid_items_and_ignores_invalid_segments() {
        let items = parse_clipboard("10 Wood, invalid, 2 Stone.", &["wood", "stone"]);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].nome, "Wood");
        assert_eq!(items[0].quantidade, 10);
        approx_eq(items[0].preco_unitario, 0.0);
        approx_eq(items[0].valor_total, 0.0);
        assert!(items[0].is_resource);
        assert_eq!(items[0].preco_input, "");

        assert_eq!(items[1].nome, "Stone");
        assert_eq!(items[1].quantidade, 2);
        assert!(items[1].is_resource);
    }

    #[test]
    fn parse_clipboard_detects_plural_and_singular_resources() {
        let items = parse_clipboard("3 woods, 4 glasses, 5 iron", &["wood", "glass"]);

        assert_eq!(items.len(), 3);
        assert!(items[0].is_resource, "woods should match wood");
        assert!(items[1].is_resource, "glasses should match glass");
        assert!(!items[2].is_resource, "iron is not in resource list");
    }

    #[test]
    fn parse_clipboard_ignores_empty_segments_between_commas() {
        let items = parse_clipboard("1 wood,, 2 stone, ,", &["wood", "stone"]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].quantidade, 1);
        assert_eq!(items[1].quantidade, 2);
    }

    #[test]
    fn parse_clipboard_ignores_leading_and_trailing_empty_segments() {
        let items = parse_clipboard(", , 1 wood,   , 2 stone,", &["wood", "stone"]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].quantidade, 1);
        assert_eq!(items[1].quantidade, 2);
    }

    #[test]
    fn parse_clipboard_ignores_segments_with_non_numeric_quantity() {
        let items = parse_clipboard("x wood, 2 stone", &["wood", "stone"]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].nome, "stone");
        assert_eq!(items[0].quantidade, 2);
    }
}
