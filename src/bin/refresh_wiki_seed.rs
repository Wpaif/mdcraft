use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[path = "../data/wiki_scraper.rs"]
mod wiki_scraper;

fn main() {
    if let Err(err) = run() {
        eprintln!("erro ao atualizar seed da wiki: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("mdcraft-seed-refresh/0.1")
        .build()?;

    let refresh = wiki_scraper::scrape_all_sources_incremental(
        &client,
        &[],
        &HashMap::new(),
        &HashMap::new(),
    )?;

    let mut items: Vec<_> = refresh
        .items
        .into_iter()
        .filter(|item| {
            item.npc_price
                .as_deref()
                .map(|p| !p.trim().is_empty())
                .unwrap_or(false)
        })
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));

    let json = serde_json::to_string_pretty(&items)?;
    let item_out_path = PathBuf::from("src/data/wiki_items_seed.json");
    fs::write(&item_out_path, format!("{json}\n"))?;

    let mut crafts = wiki_scraper::scrape_all_profession_crafts(&client)?;
    crafts.sort_by(|a, b| {
        a.profession
            .cmp(&b.profession)
            .then(a.rank.cmp(&b.rank))
            .then(a.name.cmp(&b.name))
    });

    let craft_json = serde_json::to_string_pretty(&crafts)?;
    let craft_out_path = PathBuf::from("src/data/wiki_crafts_seed.json");
    fs::write(&craft_out_path, format!("{craft_json}\n"))?;

    println!(
        "seed atualizada com {} itens em {}",
        items.len(),
        item_out_path.display()
    );
    println!(
        "seed atualizada com {} crafts em {}",
        crafts.len(),
        craft_out_path.display()
    );
    Ok(())
}
