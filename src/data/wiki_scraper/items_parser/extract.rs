use scraper::{ElementRef, Selector};

use super::price::first_price_token;
use super::text::{clean_cell_text, is_valid_item_name};

pub(in crate::data::wiki_scraper) fn extract_name_from_row(
    cells: &[ElementRef<'_>],
) -> Option<String> {
    extract_name_and_detail_path_from_row(cells).map(|(name, _)| name)
}

pub(super) fn extract_name_and_detail_path_from_row(
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

pub(super) fn extract_name_and_detail_path_from_links(
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

pub(super) fn extract_price_from_row(
    cells: &[ElementRef<'_>],
    extracted_name: &str,
) -> Option<String> {
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
