use scraper::{ElementRef, Html, Selector};

use super::text::clean_cell_text;

pub(in crate::data::wiki_scraper) fn first_price_token(text: &str) -> Option<String> {
    let mut best_with_suffix: Option<String> = None;
    let mut best_plain: Option<String> = None;

    for part in text.split_whitespace() {
        let cleaned = part
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != ',')
            .to_string();

        if cleaned.is_empty() {
            continue;
        }

        let lower = cleaned.to_lowercase();
        let valid = lower
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == 'k')
            && lower.chars().any(|c| c.is_ascii_digit());

        if valid {
            if lower.ends_with('k') {
                best_with_suffix = Some(lower);
                break;
            }

            if best_plain.is_none() {
                best_plain = Some(lower);
            }
        }
    }

    best_with_suffix
        .or(best_plain)
        .and_then(|token| normalize_npc_price_text(&token))
}

pub(in crate::data::wiki_scraper) fn normalize_npc_price_text(raw: &str) -> Option<String> {
    let value = parse_npc_price_value(raw)?;
    Some(format_npc_price_value(value))
}

pub(in crate::data::wiki_scraper) fn parse_npc_price_value(raw: &str) -> Option<f64> {
    let value = raw.trim().to_lowercase().replace(',', ".");
    if value.is_empty() {
        return None;
    }

    let parsed = if value.ends_with("kk") {
        let number = normalize_numeric_literal(value.trim_end_matches("kk"))
            .parse::<f64>()
            .ok()?;
        number * 1_000_000.0
    } else if value.ends_with('k') {
        let number = normalize_numeric_literal(value.trim_end_matches('k'))
            .parse::<f64>()
            .ok()?;
        number * 1_000.0
    } else {
        normalize_numeric_literal(&value).parse::<f64>().ok()?
    };

    if parsed.is_finite() && parsed >= 0.0 {
        Some(parsed)
    } else {
        None
    }
}

fn normalize_numeric_literal(raw: &str) -> String {
    let candidate = raw.trim();
    let dot_count = candidate.chars().filter(|&c| c == '.').count();

    if dot_count > 1 {
        return candidate.replace('.', "");
    }

    if let Some((left, right)) = candidate.split_once('.')
        && !left.is_empty()
        && right.len() == 3
        && left.chars().all(|c| c.is_ascii_digit())
    {
        return format!("{left}{right}");
    }

    candidate.to_string()
}

pub(in crate::data::wiki_scraper) fn format_npc_price_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{}kk", format_compact_decimal(value / 1_000_000.0))
    } else if value >= 1_000.0 {
        format!("{}k", format_compact_decimal(value / 1_000.0))
    } else {
        format_compact_decimal(value)
    }
}

fn format_compact_decimal(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        let mut s = format!("{:.2}", value);
        while s.contains('.') && s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
        s
    }
}

pub(in crate::data::wiki_scraper) fn extract_npc_price_from_item_detail(
    html: &str,
) -> Option<String> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("th, td").expect("cell selector should be valid");

    for row in document.select(&row_selector) {
        let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let header_text = clean_cell_text(&cells[0].text().collect::<String>()).to_lowercase();
        let header_compact: String = header_text.chars().filter(|c| c.is_alphanumeric()).collect();
        let has_npc = header_compact.contains("npc");
        let has_price =
            header_compact.contains("preco") || header_compact.contains("preço") || header_compact.contains("price");
        if !(has_npc && has_price) {
            continue;
        }

        for cell in cells.iter().skip(1) {
            let value_text = clean_cell_text(&cell.text().collect::<String>());
            if let Some(token) = first_price_token(&value_text) {
                return Some(token);
            }
        }
    }

    None
}
