use scraper::{Html, Selector};

use super::{
    clean_cell_text, extract_mediawiki_redirect_target_href, extract_name_from_row,
    extract_npc_price_from_item_detail, first_price_token, format_npc_price_value,
    is_valid_item_name, normalize_npc_price_text, parse_items_from_html,
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
    assert_eq!(first_price_token("npc 4500 coins"), Some("4.5k".to_string()));
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
                <tr><th>Item 1</th><th>Item 2</th></tr>
                <tr>
                    <td><a href="/index.php/Dog_Ear" title="Dog Ear">Dog Ear</a></td>
                    <td><a href="/index.php/Small_Tail" title="Small Tail">Small Tail</a></td>
                </tr>
            </table>
        "#;

    let items = parse_items_from_html(html, WikiSource::Loot);
    assert_eq!(items.len(), 2);
    let names: Vec<_> = items.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&"Dog Ear"));
    assert!(names.contains(&"Small Tail"));
    // No price expected for either item
    assert!(items.iter().all(|item| item.npc_price.is_none()));
}

#[test]
fn parse_item_rows_from_html_dimensional_zone_picks_only_item_column() {
    let html = r#"
            <table class="wikitable">
                <tr>
                    <th colspan="2" width="32%">Item</th>
                    <th width="31%">Dimensional Zone</th>
                </tr>
                <tr>
                    <td align="center"><img alt="Saphire3.png" src="/images/7/7e/Saphire3.png"/></td>
                    <td align="center">Sapphire</td>
                    <td align="center"><a href="/index.php/Dimensional_Zone" title="Dimensional Zone">Dimensional Zone</a></td>
                </tr>
                <tr>
                    <td align="center"><img alt="Dimensional Stone.gif" src="/images/0/09/Dimensional_Stone.gif"/></td>
                    <td align="center">Dimensional Stone</td>
                    <td align="center"><a href="/index.php/Dimensional_Zone" title="Dimensional Zone">Dimensional Zone</a></td>
                </tr>
            </table>
        "#;

    let rows = super::parse_item_rows_from_html(html, WikiSource::DimensionalZone);
    let names: Vec<_> = rows.iter().map(|r| r.item.name.as_str()).collect();
    assert_eq!(names, vec!["Sapphire", "Dimensional Stone"]);
    assert_eq!(rows[0].detail_path.as_deref(), Some("/index.php/Sapphire"));
    assert_eq!(
        rows[1].detail_path.as_deref(),
        Some("/index.php/Dimensional_Stone")
    );
}

#[test]
fn parse_item_rows_from_html_loot_index_prefers_visible_link_text() {
    let html = r#"
            <table>
                <tr><th colspan="10">Item</th></tr>
                <tr>
                    <td>
                        <a href="/index.php/Bottles_Of_Poison" title="Bottles Of Poison"><img alt="x.png" src="/images/x.png"/></a>
                        <br>
                        <a href="/index.php/Bottle_Of_Poison" title="Bottle Of Poison">Bottle Of Poison </a>
                    </td>
                </tr>
            </table>
        "#;

    let rows = super::parse_item_rows_from_html(html, WikiSource::Loot);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].item.name, "Bottle Of Poison");
    assert_eq!(rows[0].detail_path.as_deref(), Some("/index.php/Bottle_Of_Poison"));
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

#[test]
fn extract_npc_price_from_item_detail_handles_nbsp_and_accents() {
    let html = format!(
        r#"
            <table class="wikitable">
                <tr>
                    <td style="text-align:center" width="10%"><b>Preço{nbsp}NPC</b></td>
                    <td width="70%">66{nbsp}dólares</td>
                </tr>
            </table>
        "#,
        nbsp = '\u{00A0}'
    );

    let price = extract_npc_price_from_item_detail(&html);
    assert_eq!(price.as_deref(), Some("66"));
}

#[test]
fn extract_mediawiki_redirect_target_href_finds_redirect_links() {
    let html = r#"
            <div class="redirectMsg">
                <p>Redirected from</p>
                <ul class="redirectText">
                    <li><a href="/index.php/Rubber_Ball" title="Rubber Ball">Rubber Ball</a></li>
                </ul>
            </div>
        "#;

    let href = extract_mediawiki_redirect_target_href(html);
    assert_eq!(href.as_deref(), Some("/index.php/Rubber_Ball"));
}

