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
    let out_path = PathBuf::from("src/data/wiki_items_seed.json");
    fs::write(&out_path, format!("{json}\n"))?;

    println!("seed atualizada com {} itens em {}", items.len(), out_path.display());
    Ok(())
}
