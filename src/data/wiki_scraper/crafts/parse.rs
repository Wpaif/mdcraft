use scraper::{ElementRef, Html, Selector};

use super::super::{
    CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe,
    items_parser::{clean_cell_text, parse_npc_price_value},
};

pub(super) fn parse_profession_crafts_from_html(
    html: &str,
    profession: CraftProfession,
) -> Vec<ScrapedCraftRecipe> {
    let rank_sections = extract_rank_sections(html);
    let row_selector = Selector::parse("tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("td").expect("cell selector should be valid");

    let mut recipes = Vec::new();

    for (rank, section_html) in rank_sections {
        let table_htmls = extract_wikitable_html_blocks(&section_html);
        for table_html in table_htmls {
            let fragment = Html::parse_fragment(&table_html);

            for row in fragment.select(&row_selector) {
                let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
                if cells.len() < 4 {
                    continue;
                }

                let item_name = extract_craft_name_from_item_cell(cells[0]);
                if item_name.is_empty() || should_ignore_craft_name(&item_name) {
                    continue;
                }

                let ingredients = extract_ingredients_from_materials_cell(cells[3]);
                if ingredients.is_empty() {
                    continue;
                }

                recipes.push(ScrapedCraftRecipe {
                    profession,
                    rank,
                    name: item_name,
                    ingredients,
                });
            }
        }
    }

    recipes
}

fn extract_rank_sections(html: &str) -> Vec<(CraftRank, String)> {
    let mut sections = Vec::new();

    for rank in [
        CraftRank::E,
        CraftRank::D,
        CraftRank::C,
        CraftRank::B,
        CraftRank::A,
        CraftRank::S,
    ] {
        let marker = format!("id=\"Rank_{}\"", craft_rank_id(rank));
        let Some(start_idx) = html.find(&marker) else {
            continue;
        };

        let section_after_marker = &html[start_idx..];
        let section_start = section_after_marker
            .find("</h2>")
            .map(|idx| start_idx + idx + 5)
            .unwrap_or(start_idx);

        let mut section_end = html.len();
        for next_rank in [
            CraftRank::E,
            CraftRank::D,
            CraftRank::C,
            CraftRank::B,
            CraftRank::A,
            CraftRank::S,
        ] {
            if next_rank == rank {
                continue;
            }

            let next_marker = format!("id=\"Rank_{}\"", craft_rank_id(next_rank));
            if let Some(next_idx) = html[section_start..].find(&next_marker) {
                let abs_idx = section_start + next_idx;
                if abs_idx < section_end {
                    section_end = abs_idx;
                }
            }
        }

        sections.push((rank, html[section_start..section_end].to_string()));
    }

    sections
}

fn craft_rank_id(rank: CraftRank) -> &'static str {
    match rank {
        CraftRank::E => "E",
        CraftRank::D => "D",
        CraftRank::C => "C",
        CraftRank::B => "B",
        CraftRank::A => "A",
        CraftRank::S => "S",
    }
}

fn extract_wikitable_html_blocks(section_html: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cursor = 0usize;

    while let Some(rel_idx) = section_html[cursor..].find("<table") {
        let table_start = cursor + rel_idx;
        let Some(tag_end_rel) = section_html[table_start..].find('>') else {
            break;
        };
        let open_tag_end = table_start + tag_end_rel + 1;
        let open_tag = &section_html[table_start..open_tag_end];
        let open_tag_lower = open_tag.to_lowercase();

        let Some(close_rel) = section_html[open_tag_end..].find("</table>") else {
            break;
        };
        let table_end = open_tag_end + close_rel + "</table>".len();

        if open_tag_lower.contains("wikitable") {
            result.push(section_html[table_start..table_end].to_string());
        }

        cursor = table_end;
    }

    result
}

fn extract_craft_name_from_item_cell(cell: ElementRef<'_>) -> String {
    let raw = cell.text().collect::<String>();
    let cleaned = clean_cell_text(&raw);
    cleaned.trim().to_string()
}

fn should_ignore_craft_name(name: &str) -> bool {
    let normalized = name.trim().to_lowercase();
    if normalized.is_empty() {
        return true;
    }

    normalized.contains("workshop") || normalized.contains("(portátil)")
}

pub(super) fn extract_ingredients_from_materials_cell(
    cell: ElementRef<'_>,
) -> Vec<CraftIngredient> {
    let raw = clean_cell_text(&cell.text().collect::<String>());
    let tokens: Vec<&str> = raw.split_whitespace().collect();

    let mut ingredients = Vec::new();
    let mut idx = 0usize;

    while idx < tokens.len() {
        let Some(quantity) = parse_quantity_token(tokens[idx]) else {
            idx += 1;
            continue;
        };

        idx += 1;
        let start_name_idx = idx;

        while idx < tokens.len() && parse_quantity_token(tokens[idx]).is_none() {
            idx += 1;
        }

        if start_name_idx == idx {
            continue;
        }

        let name = tokens[start_name_idx..idx]
            .join(" ")
            .trim_matches(|c: char| c == '|' || c == ',' || c == ';')
            .trim()
            .to_string();

        if name.is_empty() || name == "-" || name.contains("link=]") {
            continue;
        }

        ingredients.push(CraftIngredient { name, quantity });
    }

    ingredients
}

fn parse_quantity_token(raw: &str) -> Option<f64> {
    let candidate = raw
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != ',')
        .to_lowercase();

    if candidate.is_empty() {
        return None;
    }

    parse_npc_price_value(&candidate)
}
