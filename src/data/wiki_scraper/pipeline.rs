use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use reqwest::blocking::Client;

use super::{ScrapeError, ScrapeRefreshData, ScrapedItem, WikiSource, merge, source_scrape};

type WorkerScrapeResult = Result<ScrapeRefreshData, ScrapeError>;

pub(super) fn scrape_all_sources_parallel(
    client: &Client,
    sources: &[WikiSource],
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let rx = spawn_source_scrape_workers(
        client,
        sources,
        existing_price_map,
        etag_cache,
        last_modified_cache,
    );

    collect_parallel_source_data(rx, sources.len(), etag_cache, last_modified_cache)
}

fn spawn_source_scrape_workers(
    client: &Client,
    sources: &[WikiSource],
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> mpsc::Receiver<WorkerScrapeResult> {
    let (tx, rx) = mpsc::channel();
    for &source in sources {
        let tx = tx.clone();
        let client = client.clone();
        let existing_price_map = existing_price_map.clone();
        let etag_cache = etag_cache.clone();
        let last_modified_cache = last_modified_cache.clone();

        thread::spawn(move || {
            let result = source_scrape::scrape_source_incremental_with_cache(
                &client,
                source,
                &existing_price_map,
                &etag_cache,
                &last_modified_cache,
            );
            let _ = tx.send(result);
        });
    }
    drop(tx);
    rx
}

fn collect_parallel_source_data(
    rx: mpsc::Receiver<WorkerScrapeResult>,
    worker_count: usize,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    let mut merged_etags = etag_cache.clone();
    let mut merged_last_modified = last_modified_cache.clone();
    for _ in 0..worker_count {
        let source_data = rx.recv().map_err(|err| ScrapeError::Channel {
            message: err.to_string(),
        })??;
        merge::merge_items(&mut merged, source_data.items);
        merged_etags.extend(source_data.etag_cache);
        merged_last_modified.extend(source_data.last_modified_cache);
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
    use super::collect_parallel_source_data;
    use super::super::{ScrapeError, ScrapeRefreshData, ScrapedItem, WikiSource};
    use std::collections::HashMap;
    use std::sync::mpsc;

    #[test]
    fn collect_parallel_source_data_merges_items_and_cache_metadata() {
        let (tx, rx) = mpsc::channel();

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
        .expect("send source result B should succeed");
        drop(tx);

        let existing_etags = HashMap::from([(String::from("url-existing"), String::from("etag0"))]);
        let existing_last_modified = HashMap::from([(
            String::from("url-existing"),
            String::from("Sun, 31 Dec 2023 00:00:00 GMT"),
        )]);

        let merged = collect_parallel_source_data(rx, 2, &existing_etags, &existing_last_modified)
            .expect("parallel collection should succeed");

        assert_eq!(merged.items.len(), 1);
        assert_eq!(merged.items[0].name, "Tech Data");
        assert_eq!(merged.items[0].npc_price.as_deref(), Some("1k"));
        assert!(merged.items[0].sources.contains(&WikiSource::Loot));
        assert!(merged.items[0].sources.contains(&WikiSource::Nightmare));

        assert_eq!(merged.etag_cache.get("url-existing").map(String::as_str), Some("etag0"));
        assert_eq!(merged.etag_cache.get("url-a").map(String::as_str), Some("etag-a"));
        assert_eq!(merged.etag_cache.get("url-b").map(String::as_str), Some("etag-b"));
        assert_eq!(
            merged
                .last_modified_cache
                .get("url-existing")
                .map(String::as_str),
            Some("Sun, 31 Dec 2023 00:00:00 GMT")
        );
    }

    #[test]
    fn collect_parallel_source_data_returns_channel_error_when_sender_drops() {
        let (tx, rx) = mpsc::channel::<Result<ScrapeRefreshData, ScrapeError>>();
        drop(tx);

        let err = collect_parallel_source_data(rx, 1, &HashMap::new(), &HashMap::new())
            .expect_err("collect should fail if channel closes before worker data");

        assert!(matches!(err, ScrapeError::Channel { .. }));
    }
}
