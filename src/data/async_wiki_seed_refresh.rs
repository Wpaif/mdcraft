use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn start_async_wiki_seed_refresh(interval_secs: u64) {
    let client = Client::builder()
        .user_agent("mdcraft-async-seed-refresh/0.1")
        .build()
        .expect("Failed to build HTTP client");
    let client = Arc::new(client);
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    tokio::spawn(async move {
        while *running_clone.lock().await {
            if let Err(e) = refresh_wiki_seeds(client.clone()).await {
                eprintln!("Erro ao atualizar seeds da wiki: {e}");
            }
            sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

async fn refresh_wiki_seeds(client: Arc<Client>) -> Result<(), Box<dyn std::error::Error>> {
    let refresh = crate::data::wiki_scraper::scrape_all_sources_incremental_async(
        &client,
        &[],
        &std::collections::HashMap::new(),
        &std::collections::HashMap::new(),
    )
    .await?;

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
    let item_out_path = std::path::PathBuf::from("src/data/wiki_items_seed.json");
    std::fs::write(&item_out_path, format!("{json}\n"))?;

    // Comparar com o arquivo estático e salvar diff de preços
    let static_path = std::path::PathBuf::from("src/data/wiki_items_seed_static.json");
    if let (Ok(static_data), Ok(new_data)) = (
        std::fs::read_to_string(&static_path),
        std::fs::read_to_string(&item_out_path),
    ) {
        let static_items: Vec<crate::data::wiki_scraper::ScrapedItem> =
            serde_json::from_str(&static_data).unwrap_or_default();
        let new_items: Vec<crate::data::wiki_scraper::ScrapedItem> =
            serde_json::from_str(&new_data).unwrap_or_default();
        use std::collections::HashMap;
        let static_map: HashMap<_, _> = static_items
            .iter()
            .map(|i| (i.name.to_lowercase(), i.npc_price.clone()))
            .collect();
        let new_map: HashMap<_, _> = new_items
            .iter()
            .map(|i| (i.name.to_lowercase(), i.npc_price.clone()))
            .collect();
        let mut diffs = Vec::new();
        for (name, new_price) in &new_map {
            let static_price = static_map.get(name).cloned().flatten();
            if static_price != *new_price {
                diffs.push(serde_json::json!({
                    "name": name,
                    "old_price": static_price,
                    "new_price": new_price
                }));
            }
        }
        let diff_path = std::path::PathBuf::from("src/data/wiki_items_seed_diff.json");
        let diff_json = serde_json::to_string_pretty(&diffs)?;
        std::fs::write(&diff_path, format!("{diff_json}\n"))?;
        println!(
            "Diff de preços salvo em {} ({} mudanças)",
            diff_path.display(),
            diffs.len()
        );
    }

    // Crafts async
    let mut crafts =
        crate::data::wiki_scraper::crafts::scrape_all_profession_crafts_async(&client).await?;
    crafts.sort_by(|a, b| {
        a.profession
            .cmp(&b.profession)
            .then(a.rank.cmp(&b.rank))
            .then(a.name.cmp(&b.name))
    });

    let craft_json = serde_json::to_string_pretty(&crafts)?;
    let craft_out_path = std::path::PathBuf::from("src/data/wiki_crafts_seed.json");
    std::fs::write(&craft_out_path, format!("{craft_json}\n"))?;

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
