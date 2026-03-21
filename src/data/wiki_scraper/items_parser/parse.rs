use scraper::{ElementRef, Html, Selector};

use super::extract::{
    extract_name_and_detail_path_from_links, extract_price_from_row,
};
use super::price::first_price_token;
use super::text::{clean_cell_text, is_valid_item_name, normalize_key};
use super::types::ParsedItemRow;
use super::super::{ScrapedItem, WikiSource};

pub(in crate::data::wiki_scraper) fn parse_items_from_html(
    html: &str,
    source: WikiSource,
) -> Vec<ScrapedItem> {
    parse_item_rows_from_html(html, source)
        .into_iter()
        .map(|row| row.item)
        .collect()
}

pub(in crate::data::wiki_scraper) fn parse_item_rows_from_html(
    html: &str,
    source: WikiSource,
) -> Vec<ParsedItemRow> {
    if matches!(source, WikiSource::Loot | WikiSource::Nightmare) {
        let index_rows = parse_loot_like_index_item_rows(html, source);
        if !index_rows.is_empty() {
            return index_rows;
        }
    }

    let document = Html::parse_document(html);
    // Aceita tanto <table> quanto <table class="wikitable">
    let table_selector = Selector::parse("table").expect("table selector should be valid");
    let row_selector = Selector::parse("tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("td").expect("cell selector should be valid");
    let header_cell_selector =
        Selector::parse("th, td").expect("header cell selector should be valid");

    let mut result = Vec::new();

    for table in document.select(&table_selector) {
        // Filtra tabelas irrelevantes (evita capturar links de "Veja mais", navegação, etc).
        let header_compact = table
            .select(&row_selector)
            .next()
            .map(|row| {
                row.select(&header_cell_selector)
                    .map(|cell| clean_cell_text(&cell.text().collect::<String>()))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        match source {
            WikiSource::DimensionalZone => {
                // A tabela relevante tem cabeçalho "Item" + "Dimensional Zone".
                if !(header_compact.contains("item")
                    && header_compact.contains("dimensionalzone"))
                {
                    continue;
                }

                let mut rows = table.select(&row_selector).peekable();
                if rows.peek().is_some() {
                    rows.next(); // header
                }

                for row in rows {
                    let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
                    if cells.len() < 2 {
                        continue;
                    }

                    let name = clean_cell_text(&cells[1].text().collect::<String>());
                    if !is_valid_item_name(&name) {
                        continue;
                    }

                    let detail_path = if name.is_ascii() {
                        Some(format!("/index.php/{}", name.trim().replace(' ', "_")))
                    } else {
                        None
                    };

                    result.push(ParsedItemRow {
                        item: ScrapedItem {
                            name,
                            npc_price: None,
                            sources: vec![source],
                        },
                        detail_path,
                    });
                }

                continue;
            }
            WikiSource::Loot | WikiSource::Nightmare => {
                // Nesses índices, os itens relevantes estão nas tabelas cujo cabeçalho contém "Item/Itens".
                if !(header_compact.contains("item") || header_compact.contains("itens")) {
                    continue;
                }
            }
        }

        let mut rows = table.select(&row_selector).peekable();
        // Pular o cabeçalho (primeira linha)
        if rows.peek().is_some() {
            rows.next();
        }
        for row in rows {
            let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
            if cells.is_empty() {
                continue;
            }

            if cells.len() > 1 {
                // Identifica todas as células de item e de preço
                let mut item_cells = Vec::new();
                let mut price_cell: Option<String> = None;
                for cell in &cells {
                    let text = clean_cell_text(&cell.text().collect::<String>());
                    if let Some((name, detail_path)) =
                        extract_name_and_detail_path_from_links(*cell)
                    {
                        if is_valid_item_name(&name) && first_price_token(&text).is_none() {
                            item_cells.push((name, detail_path));
                        }
                    } else if is_valid_item_name(&text) && first_price_token(&text).is_none() {
                        item_cells.push((text, None));
                    } else if let Some(token) = first_price_token(&text) {
                        price_cell = Some(token);
                    }
                }
                // Se só há um item e um preço, associa o preço; senão, cada item sem preço
                for (name, detail_path) in &item_cells {
                    let npc_price = if item_cells.len() == 1 {
                        price_cell.clone()
                    } else {
                        None
                    };
                    result.push(ParsedItemRow {
                        item: ScrapedItem {
                            name: name.clone(),
                            npc_price,
                            sources: vec![source.clone()],
                        },
                        detail_path: detail_path.clone(),
                    });
                }
            } else {
                // Linha de uma célula: pode ser item + preço, ou só item
                let cell = &cells[0];
                if let Some((name, detail_path)) =
                    extract_name_and_detail_path_from_links(*cell)
                {
                    if is_valid_item_name(&name) {
                        let npc_price = extract_price_from_row(&cells, &name);
                        result.push(ParsedItemRow {
                            item: ScrapedItem {
                                name,
                                npc_price,
                                sources: vec![source.clone()],
                            },
                            detail_path,
                        });
                    }
                } else {
                    let text = clean_cell_text(&cell.text().collect::<String>());
                    if is_valid_item_name(&text) {
                        let npc_price = extract_price_from_row(&cells, &text);
                        result.push(ParsedItemRow {
                            item: ScrapedItem {
                                name: text,
                                npc_price,
                                sources: vec![source.clone()],
                            },
                            detail_path: None,
                        });
                    }
                }
            }
        }
    }

    result
}

fn parse_loot_like_index_item_rows(html: &str, source: WikiSource) -> Vec<ParsedItemRow> {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").expect("table selector should be valid");
    let row_selector = Selector::parse("tr").expect("row selector should be valid");
    let header_cell_selector =
        Selector::parse("th, td").expect("header cell selector should be valid");
    let cell_selector = Selector::parse("td").expect("cell selector should be valid");
    let link_selector = Selector::parse("a[href][title]").expect("link selector should be valid");

    let mut result = Vec::new();
    let mut seen_keys = std::collections::HashSet::<String>::new();

    for table in document.select(&table_selector) {
        let header_compact = table
            .select(&row_selector)
            .next()
            .map(|row| {
                row.select(&header_cell_selector)
                    .map(|cell| clean_cell_text(&cell.text().collect::<String>()))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        // Nas páginas de índice, as tabelas úteis começam com um cabeçalho "Item".
        if header_compact != "item"
            && !header_compact.ends_with("item")
            && !header_compact.contains("item")
        {
            continue;
        }

        let mut rows = table.select(&row_selector).peekable();
        if rows.peek().is_some() {
            rows.next(); // header
        }

        for row in rows {
            for cell in row.select(&cell_selector) {
                let mut best: Option<(String, String)> = None;

                for link in cell.select(&link_selector) {
                    let Some(title) = link.value().attr("title") else {
                        continue;
                    };
                    if title.starts_with("Arquivo:") || title.starts_with("File:") {
                        continue;
                    }
                    let Some(href) = link.value().attr("href") else {
                        continue;
                    };

                    let visible_text = clean_cell_text(&link.text().collect::<String>());
                    let visible_is_valid = is_valid_item_name(&visible_text);
                    let candidate_name = if visible_is_valid {
                        visible_text.clone()
                    } else {
                        clean_cell_text(title)
                    };

                    if !is_valid_item_name(&candidate_name) {
                        continue;
                    }

                    // Prefere link com texto visível (normalmente o nome do item, depois do <br>).
                    let is_visible = visible_is_valid;
                    let is_better = match &best {
                        None => true,
                        Some((prev_name, _)) => {
                            (is_visible && prev_name != &candidate_name)
                                || (!prev_name.is_empty()
                                    && prev_name.len() < candidate_name.len())
                        }
                    };
                    if is_better {
                        best = Some((candidate_name, href.to_string()));
                    }
                }

                let Some((name, href)) = best else {
                    continue;
                };

                let key = normalize_key(&name);
                if !seen_keys.insert(key) {
                    continue;
                }

                result.push(ParsedItemRow {
                    item: ScrapedItem {
                        name,
                        npc_price: None,
                        sources: vec![source],
                    },
                    detail_path: Some(href),
                });
            }
        }
    }

    result
}
