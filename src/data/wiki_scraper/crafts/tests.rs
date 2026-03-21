use scraper::{Html, Selector};

use super::{parse_profession_crafts_from_html, parse::extract_ingredients_from_materials_cell};
use super::super::{CraftProfession, CraftRank};

#[test]
fn parse_profession_crafts_from_html_extracts_ranked_recipes() {
    let html = r#"
            <h2><span class="mw-headline" id="Rank_E">Rank E</span></h2>
            <table class="wikitable sortable">
                <tr><th>Item</th><th>Habilidade</th><th>Tempo</th><th>Materiais</th></tr>
                <tr>
                    <td><img alt="Poke-ball(1).png"/> Poke Ball (100x)</td>
                    <td>Skill 0</td>
                    <td>1 Minuto</td>
                    <td><img alt="Apricorn.png"/> 1 Apricorn <br/> <img alt="Screw.png"/> 80 Screw</td>
                </tr>
                <tr>
                    <td><img alt="Workshop D.png"/> Workshop D</td>
                    <td>Skill 20</td>
                    <td>2 Horas</td>
                    <td><img alt="Diamond.png"/> 1 Diamond</td>
                </tr>
            </table>
            <h2><span class="mw-headline" id="Rank_D">Rank D</span></h2>
            <table class="wikitable sortable">
                <tr><th>Item</th><th>Habilidade</th><th>Tempo</th><th>Materiais</th></tr>
                <tr>
                    <td>Great Ball (100x)</td>
                    <td>Skill 20</td>
                    <td>6 Minutos</td>
                    <td>1 Apricorn 250 Screw 4 Iron Bar</td>
                </tr>
            </table>
        "#;

    let recipes = parse_profession_crafts_from_html(html, CraftProfession::Engineer);
    assert_eq!(recipes.len(), 2);

    assert_eq!(recipes[0].profession, CraftProfession::Engineer);
    assert_eq!(recipes[0].rank, CraftRank::E);
    assert_eq!(recipes[0].name, "Poke Ball (100x)");
    assert_eq!(recipes[0].ingredients.len(), 2);
    assert_eq!(recipes[0].ingredients[0].name, "Apricorn");
    assert_eq!(recipes[0].ingredients[0].quantity, 1.0);

    assert_eq!(recipes[1].rank, CraftRank::D);
    assert_eq!(recipes[1].name, "Great Ball (100x)");
    assert_eq!(recipes[1].ingredients.len(), 3);
}

#[test]
fn extract_ingredients_from_materials_cell_parses_thousand_and_k_tokens() {
    let html = Html::parse_fragment(
        r#"<table><tr><td>1.000 Screw 2.5k Iron Ore 3 Diamond</td></tr></table>"#,
    );
    let row_selector = Selector::parse("tr").expect("valid row selector");
    let cell_selector = Selector::parse("td").expect("valid cell selector");
    let row = html.select(&row_selector).next().expect("row should exist");
    let cell = row
        .select(&cell_selector)
        .next()
        .expect("cell should exist");

    let ingredients = extract_ingredients_from_materials_cell(cell);
    assert_eq!(ingredients.len(), 3);
    assert_eq!(ingredients[0].name, "Screw");
    assert_eq!(ingredients[0].quantity, 1000.0);
    assert_eq!(ingredients[1].name, "Iron Ore");
    assert_eq!(ingredients[1].quantity, 2500.0);
    assert_eq!(ingredients[2].name, "Diamond");
    assert_eq!(ingredients[2].quantity, 3.0);
}

