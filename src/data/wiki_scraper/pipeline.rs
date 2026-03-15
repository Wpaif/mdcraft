use reqwest::Client;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;

use super::{ScrapeError, ScrapeRefreshData, ScrapedItem, WikiSource, merge};

type WorkerScrapeResult = Result<ScrapeRefreshData, ScrapeError>;

pub async fn scrape_all_sources_parallel_async(
    client: &Client,
    sources: &[WikiSource],
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let rx = spawn_source_scrape_workers_async(
        client,
        sources,
        existing_price_map,
        etag_cache,
        last_modified_cache,
    )
    .await;
    collect_parallel_source_data_async(rx, sources.len(), etag_cache, last_modified_cache).await
}

async fn spawn_source_scrape_workers_async(
    client: &Client,
    sources: &[WikiSource],
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> mpsc::Receiver<WorkerScrapeResult> {
    let (tx, rx) = mpsc::channel(sources.len());
    for &source in sources {
        let tx = tx.clone();
        let client = client.clone();
        let existing_price_map = existing_price_map.clone();
        let etag_cache = etag_cache.clone();
        let last_modified_cache = last_modified_cache.clone();
        task::spawn(async move {
            let result = super::source_scrape::scrape_source_incremental_with_cache_async(
                &client,
                source,
                &existing_price_map,
                &etag_cache,
                &last_modified_cache,
            )
            .await;
            if let Err(e) = tx.send(result).await {
                eprintln!(
                    "[async-scrape][ERRO] Falha ao enviar resultado do worker para {:?}: {e}",
                    source
                );
            }
        });
    }
    rx
}

async fn collect_parallel_source_data_async(
    rx: mpsc::Receiver<WorkerScrapeResult>,
    worker_count: usize,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    let mut merged_etags = etag_cache.clone();
    let mut merged_last_modified = last_modified_cache.clone();
    let mut rx = rx;
    for i in 0..worker_count {
        match rx.recv().await {
            Some(Ok(source_data)) => {
                merge::merge_items(&mut merged, source_data.items);
                merged_etags.extend(source_data.etag_cache);
                merged_last_modified.extend(source_data.last_modified_cache);
            }
            Some(Err(e)) => {
                eprintln!("[async-scrape][ERRO] Worker {} retornou erro: {:?}", i, e);
                return Err(e);
            }
            None => {
                eprintln!(
                    "[async-scrape][ERRO] Canal de comunicação fechado antes do esperado (worker {}).",
                    i
                );
                return Err(ScrapeError::Channel {
                    message: format!("channel closed (worker {})", i),
                });
            }
        }
    }
    let items = merge::finalize_scraped_items(merged.into_values().collect());
    Ok(ScrapeRefreshData {
        items,
        etag_cache: merged_etags,
        last_modified_cache: merged_last_modified,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{ScrapeError, ScrapeRefreshData, ScrapedItem, WikiSource};
    use super::collect_parallel_source_data_async;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn collect_parallel_source_data_merges_items_and_cache_metadata() {
        let (tx, rx) = mpsc::channel(2);

        tx.send(Ok(ScrapeRefreshData {
            items: vec![ScrapedItem {
                name: "Tech Data".to_string(),
                npc_price: Some("1k".to_string()),
                sources: vec![WikiSource::Loot],
            }],
            etag_cache: HashMap::from([(String::from("url-a"), String::from("etag-a"))]),
            last_modified_cache: HashMap::from([(
                String::from("url-a"),
                String::from("Mon, 01 Jan 2024 00:00:00 GMT"),
            )]),
        }))
        .await
        .expect("send source result A should succeed");

        tx.send(Ok(ScrapeRefreshData {
            items: vec![ScrapedItem {
                name: "Tech Data".to_string(),
                npc_price: None,
                sources: vec![WikiSource::Nightmare],
            }],
            etag_cache: HashMap::from([(String::from("url-b"), String::from("etag-b"))]),
            last_modified_cache: HashMap::from([(
                String::from("url-b"),
                String::from("Tue, 02 Jan 2024 00:00:00 GMT"),
            )]),
        }))
        .await
        .expect("send source result B should succeed");
        drop(tx);

        let existing_etags = HashMap::from([(String::from("url-existing"), String::from("etag0"))]);
        let existing_last_modified = HashMap::from([(
            String::from("url-existing"),
            String::from("Sun, 31 Dec 2023 00:00:00 GMT"),
        )]);

        let merged =
            collect_parallel_source_data_async(rx, 2, &existing_etags, &existing_last_modified)
                .await
                .expect("parallel collection should succeed");

        assert_eq!(merged.items.len(), 1);
        assert_eq!(merged.items[0].name, "Tech Data");
        assert_eq!(merged.items[0].npc_price.as_deref(), Some("1k"));
        assert!(merged.items[0].sources.contains(&WikiSource::Loot));
        assert!(merged.items[0].sources.contains(&WikiSource::Nightmare));

        assert_eq!(
            merged.etag_cache.get("url-existing").map(String::as_str),
            Some("etag0")
        );
        assert_eq!(
            merged.etag_cache.get("url-a").map(String::as_str),
            Some("etag-a")
        );
        assert_eq!(
            merged.etag_cache.get("url-b").map(String::as_str),
            Some("etag-b")
        );
        assert_eq!(
            merged
                .last_modified_cache
                .get("url-existing")
                .map(String::as_str),
            Some("Sun, 31 Dec 2023 00:00:00 GMT")
        );
    }

    #[tokio::test]
    async fn collect_parallel_source_data_returns_channel_error_when_sender_drops() {
        let (tx, rx) = mpsc::channel(1);
        drop(tx);

        let err = collect_parallel_source_data_async(rx, 1, &HashMap::new(), &HashMap::new())
            .await
            .expect_err("collect should fail if channel closes before worker data");

        assert!(matches!(err, ScrapeError::Channel { .. }));
    }
}
