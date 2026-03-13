use scraper::{ElementRef, Html, Selector};

use super::{ScrapedItem, WikiSource};

pub(super) struct ParsedItemRow {
    pub(super) item: ScrapedItem,
    pub(super) detail_path: Option<String>,
}

pub(super) fn normalize_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub(super) fn parse_items_from_html(html: &str, source: WikiSource) -> Vec<ScrapedItem> {
    parse_item_rows_from_html(html, source)
        .into_iter()
        .map(|row| row.item)
        .collect()
}

pub(super) fn parse_item_rows_from_html(html: &str, source: WikiSource) -> Vec<ParsedItemRow> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("td").expect("cell selector should be valid");

    let mut result = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let linked_items: Vec<(String, Option<String>)> = cells
            .iter()
            .filter_map(|cell| extract_name_and_detail_path_from_links(*cell))
            .collect();

        if !linked_items.is_empty() {
            // Loot tables often place many items in a single row. Parse one item per cell.
            for (name, detail_path) in linked_items {
                let npc_price = if cells.len() <= 2 {
                    extract_price_from_row(&cells, &name)
                } else {
                    None
                };

                result.push(ParsedItemRow {
                    item: ScrapedItem {
                        name,
                        npc_price,
                        sources: vec![source],
                    },
                    detail_path,
                });
            }
            continue;
        }

        let Some((name, detail_path)) = extract_name_and_detail_path_from_row(&cells) else {
            continue;
        };

        let npc_price = extract_price_from_row(&cells, &name);
        result.push(ParsedItemRow {
            item: ScrapedItem {
                name,
                npc_price,
                sources: vec![source],
            },
            detail_path,
        });
    }

    result
}

pub(super) fn extract_name_from_row(cells: &[ElementRef<'_>]) -> Option<String> {
    extract_name_and_detail_path_from_row(cells).map(|(name, _)| name)
}

fn extract_name_and_detail_path_from_row(
    cells: &[ElementRef<'_>],
) -> Option<(String, Option<String>)> {
    for cell in cells {
        if let Some(name_and_path) = extract_name_and_detail_path_from_links(*cell) {
            return Some(name_and_path);
        }
    }

    for cell in cells {
        let text = clean_cell_text(&cell.text().collect::<String>());
        if is_valid_item_name(&text) {
            return Some((text, None));
        }
    }

    None
}

fn extract_name_and_detail_path_from_links(
    cell: ElementRef<'_>,
) -> Option<(String, Option<String>)> {
    let link_selector = Selector::parse("a[title]").expect("link selector should be valid");

    for link in cell.select(&link_selector) {
        let Some(title) = link.value().attr("title") else {
            continue;
        };

        if title.starts_with("Arquivo:") || title.starts_with("File:") {
            continue;
        }

        let normalized = clean_cell_text(title);
        if is_valid_item_name(&normalized) {
            let detail_path = link.value().attr("href").map(ToString::to_string);
            return Some((normalized, detail_path));
        }
    }

    None
}

fn extract_price_from_row(cells: &[ElementRef<'_>], extracted_name: &str) -> Option<String> {
    for cell in cells {
        let text = clean_cell_text(&cell.text().collect::<String>());

        if text.eq_ignore_ascii_case(extracted_name) {
            continue;
        }

        if let Some(price) = first_price_token(&text) {
            return Some(price);
        }
    }

    None
}

pub(super) fn first_price_token(text: &str) -> Option<String> {
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

pub(super) fn normalize_npc_price_text(raw: &str) -> Option<String> {
    let value = parse_npc_price_value(raw)?;
    Some(format_npc_price_value(value))
}

pub(super) fn parse_npc_price_value(raw: &str) -> Option<f64> {
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

pub(super) fn format_npc_price_value(value: f64) -> String {
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

pub(super) fn extract_npc_price_from_item_detail(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("th, td").expect("cell selector should be valid");

    for row in document.select(&row_selector) {
        let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let header_text = clean_cell_text(&cells[0].text().collect::<String>()).to_lowercase();
        if !header_text.contains("preço npc") && !header_text.contains("preco npc") {
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

pub(super) fn clean_cell_text(text: &str) -> String {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let mut kept = Vec::new();

    for (idx, part) in parts.iter().enumerate() {
        if looks_like_media_filename(part) {
            continue;
        }

        let next_is_media = parts
            .get(idx + 1)
            .map(|next| looks_like_media_filename(next))
            .unwrap_or(false);

        // Handles names split before extension like: "Ancient Wire.png Ancient Wire".
        if next_is_media {
            continue;
        }

        kept.push(*part);
    }

    kept.join(" ").trim().to_string()
}

fn looks_like_media_filename(part: &str) -> bool {
    let lower = part.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
}

pub(super) fn is_valid_item_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let lower = name.to_lowercase();
    if lower == "item" || lower == "itens" || lower == "nightmare world" {
        return false;
    }

    lower.chars().any(|c| c.is_alphabetic())
}

#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    use super::{
        clean_cell_text, extract_name_from_row, extract_npc_price_from_item_detail,
        first_price_token, format_npc_price_value, is_valid_item_name, normalize_npc_price_text,
        parse_items_from_html,
    };
    use super::super::WikiSource;

    #[test]
    fn clean_cell_text_removes_media_tokens() {
        let cleaned = clean_cell_text("Ancient Wire.png Ancient Wire");
        assert_eq!(cleaned, "Ancient Wire");
    }

    #[test]
    fn first_price_token_extracts_k_or_plain_number() {
        assert_eq!(first_price_token("Preco: 12k"), Some("12k".to_string()));
        assert_eq!(
            first_price_token("npc 4500 coins"),
            Some("4.5k".to_string())
        );
        assert_eq!(
            first_price_token("100.000 dolares (100K)"),
            Some("100k".to_string())
        );
        assert_eq!(first_price_token("sem preco"), None);
    }

    #[test]
    fn normalize_npc_price_text_canonicalizes_mixed_formats() {
        assert_eq!(normalize_npc_price_text("2.500"), Some("2.5k".to_string()));
        assert_eq!(normalize_npc_price_text("132.00"), Some("132".to_string()));
        assert_eq!(normalize_npc_price_text("100K"), Some("100k".to_string()));
        assert_eq!(normalize_npc_price_text("1.5KK"), Some("1.5kk".to_string()));
    }

    #[test]
    fn format_npc_price_value_uses_compact_suffixes() {
        assert_eq!(format_npc_price_value(2500.0), "2.5k");
        assert_eq!(format_npc_price_value(100000.0), "100k");
        assert_eq!(format_npc_price_value(0.5), "0.5");
    }

    #[test]
    fn is_valid_item_name_filters_headers() {
        assert!(!is_valid_item_name("Item"));
        assert!(!is_valid_item_name("Itens"));
        assert!(is_valid_item_name("Ancient Wire"));
    }

    #[test]
    fn extract_name_from_row_prefers_link_title() {
        let html = Html::parse_fragment(
            r#"<table><tr>
                <td><a title="Arquivo:Ancient_Wire.png">img</a><a title="Ancient Wire">Ancient Wire</a></td>
                <td>12k</td>
            </tr></table>"#,
        );

        let row_selector = Selector::parse("tr").expect("valid row selector");
        let cell_selector = Selector::parse("td").expect("valid cell selector");
        let row = html.select(&row_selector).next().expect("row should exist");
        let cells = row.select(&cell_selector).collect::<Vec<_>>();

        let extracted = extract_name_from_row(&cells);
        assert_eq!(extracted, Some("Ancient Wire".to_string()));
    }

    #[test]
    fn parse_items_from_html_supports_single_cell_tables() {
        let html = r#"
            <table>
                <tr><th>Item</th></tr>
                <tr><td>Ancient Wire.png Ancient Wire</td></tr>
                <tr><td>Gear Nose.png Gear Nose</td></tr>
            </table>
        "#;

        let items = parse_items_from_html(html, WikiSource::Nightmare);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Ancient Wire");
        assert_eq!(items[0].npc_price, None);
        assert_eq!(items[1].name, "Gear Nose");
    }

    #[test]
    fn parse_items_from_html_extracts_optional_price() {
        let html = r#"
            <table>
                <tr><th>Item</th><th>Preco</th></tr>
                <tr>
                    <td><a title="Ancient Wire">Ancient Wire</a></td>
                    <td>12k</td>
                </tr>
            </table>
        "#;

        let items = parse_items_from_html(html, WikiSource::Loot);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Ancient Wire");
        assert_eq!(items[0].npc_price, Some("12k".to_string()));
    }

    #[test]
    fn parse_items_from_html_extracts_multiple_items_per_row() {
        let html = r#"
            <table>
                <tr>
                    <td><a href="/index.php/Dog_Ear" title="Dog Ear">Dog Ear</a></td>
                    <td><a href="/index.php/Small_Tail" title="Small Tail">Small Tail</a></td>
                </tr>
            </table>
        "#;

        let items = parse_items_from_html(html, WikiSource::Loot);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Dog Ear");
        assert_eq!(items[1].name, "Small Tail");
        assert!(items.iter().all(|item| item.npc_price.is_none()));
    }

    #[test]
    fn extract_npc_price_from_item_detail_finds_price_row() {
        let html = r#"
            <table>
                <tr><td><b>Preco NPC</b></td><td>100.000 dolares (100K)</td></tr>
            </table>
        "#;

        let price = extract_npc_price_from_item_detail(html);
        assert_eq!(price.as_deref(), Some("100k"));
    }
}
