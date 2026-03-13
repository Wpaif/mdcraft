use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};

use super::{
    ScrapeError, ScrapeRefreshData, WikiSource, items_parser, merge,
};

const DETAIL_FETCH_WORKERS: usize = 3;
const DETAIL_FETCH_BASE_DELAY_MS: u64 = 220;
const DETAIL_FETCH_JITTER_MS: u64 = 180;

trait HttpCacheClient: Send + Sync {
    fn fetch_url_with_cache(
        &self,
        url: &str,
        etag: Option<&String>,
        last_modified: Option<&String>,
    ) -> Result<CachedFetch, String>;
}

struct ReqwestHttpCacheClient {
    client: Client,
}

impl ReqwestHttpCacheClient {
    fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
        }
    }
}

impl HttpCacheClient for ReqwestHttpCacheClient {
    fn fetch_url_with_cache(
        &self,
        url: &str,
        etag: Option<&String>,
        last_modified: Option<&String>,
    ) -> Result<CachedFetch, String> {
        fetch_url_with_cache_reqwest(&self.client, url, etag, last_modified)
            .map_err(|err| err.to_string())
    }
}

pub(super) fn scrape_source_incremental_with_cache(
    client: &Client,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    scrape_source_incremental_with_http(
        Arc::new(ReqwestHttpCacheClient::new(client)),
        source,
        existing_price_map,
        etag_cache,
        last_modified_cache,
    )
}

fn scrape_source_incremental_with_http(
    http_client: Arc<dyn HttpCacheClient>,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let mut etag_updates = HashMap::new();
    let mut last_modified_updates = HashMap::new();
    let source_url = source.url().to_string();

    let html = match http_client.fetch_url_with_cache(
        &source_url,
        etag_cache.get(&source_url),
        last_modified_cache.get(&source_url),
    ) {
        Err(err) => {
            return Err(ScrapeError::Request {
                source,
                message: err,
            });
        }
        Ok(CachedFetch::NotModified) => {
            return Ok(ScrapeRefreshData {
                items: Vec::new(),
                etag_cache: etag_updates,
                last_modified_cache: last_modified_updates,
            });
        }
        Ok(CachedFetch::Modified {
            html,
            etag,
            last_modified,
        }) => {
            if let Some(etag) = etag {
                etag_updates.insert(source_url.clone(), etag);
            }
            if let Some(last_modified) = last_modified {
                last_modified_updates.insert(source_url.clone(), last_modified);
            }
            html
        }
    };

    let mut rows = items_parser::parse_item_rows_from_html(&html, source);
    for row in &mut rows {
        if row.item.npc_price.is_some() {
            continue;
        }

        if let Some(price) = existing_price_map.get(&items_parser::normalize_key(&row.item.name)) {
            row.item.npc_price = Some(price.clone());
        }
    }

    fill_missing_prices_from_details(
        Arc::clone(&http_client),
        &mut rows,
        existing_price_map,
        etag_cache,
        last_modified_cache,
        &mut etag_updates,
        &mut last_modified_updates,
    );

    Ok(ScrapeRefreshData {
        items: merge::retain_items_with_npc_price(rows.into_iter().map(|row| row.item).collect()),
        etag_cache: etag_updates,
        last_modified_cache: last_modified_updates,
    })
}

fn fill_missing_prices_from_details(
    http_client: Arc<dyn HttpCacheClient>,
    rows: &mut [items_parser::ParsedItemRow],
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
    etag_updates: &mut HashMap<String, String>,
    last_modified_updates: &mut HashMap<String, String>,
) {
    let mut seen = HashSet::new();
    let detail_paths: Vec<String> = rows
        .iter()
        .filter(|row| row.item.npc_price.is_none())
        .filter(|row| !existing_price_map.contains_key(&items_parser::normalize_key(&row.item.name)))
        .filter_map(|row| row.detail_path.clone())
        .map(|path| resolve_wiki_url(&path))
        .filter(|url| seen.insert(url.clone()))
        .collect();

    if detail_paths.is_empty() {
        return;
    }

    let queue = Arc::new(Mutex::new(VecDeque::from(detail_paths)));
    let prices = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let etag_cache = Arc::new(etag_cache.clone());
    let last_modified_cache = Arc::new(last_modified_cache.clone());
    let detail_etag_updates = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let detail_last_modified_updates = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let worker_count = DETAIL_FETCH_WORKERS.max(1);

    let mut workers = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let prices = Arc::clone(&prices);
        let etag_cache = Arc::clone(&etag_cache);
        let last_modified_cache = Arc::clone(&last_modified_cache);
        let detail_etag_updates = Arc::clone(&detail_etag_updates);
        let detail_last_modified_updates = Arc::clone(&detail_last_modified_updates);
        let http_client = Arc::clone(&http_client);

        workers.push(thread::spawn(move || {
            loop {
                let path = {
                    let mut guard = queue.lock().expect("detail queue mutex poisoned");
                    guard.pop_front()
                };

                let Some(path) = path else {
                    break;
                };

                thread::sleep(polite_delay_for_path(&path));

                let detail_fetch = http_client.fetch_url_with_cache(
                    &path,
                    etag_cache.get(&path),
                    last_modified_cache.get(&path),
                );
                let Ok(detail_fetch) = detail_fetch else {
                    continue;
                };

                let detail_html = match detail_fetch {
                    CachedFetch::NotModified => continue,
                    CachedFetch::Modified {
                        html,
                        etag,
                        last_modified,
                    } => {
                        if let Some(etag) = etag {
                            let mut etag_guard = detail_etag_updates
                                .lock()
                                .expect("detail etag updates mutex poisoned");
                            etag_guard.insert(path.clone(), etag);
                        }
                        if let Some(last_modified) = last_modified {
                            let mut last_mod_guard = detail_last_modified_updates
                                .lock()
                                .expect("detail last-modified updates mutex poisoned");
                            last_mod_guard.insert(path.clone(), last_modified);
                        }
                        html
                    }
                };

                let Some(price) = items_parser::extract_npc_price_from_item_detail(&detail_html) else {
                    continue;
                };

                let mut guard = prices.lock().expect("detail prices mutex poisoned");
                guard.insert(path, price);
            }
        }));
    }

    for worker in workers {
        let _ = worker.join();
    }

    let price_map = prices.lock().expect("detail prices mutex poisoned");
    for row in rows {
        if row.item.npc_price.is_some() {
            continue;
        }

        let Some(path) = row.detail_path.as_ref() else {
            continue;
        };
        let resolved = resolve_wiki_url(path);

        if let Some(price) = price_map.get(&resolved) {
            row.item.npc_price = Some(price.clone());
        }
    }

    let detail_etag_updates = detail_etag_updates
        .lock()
        .expect("detail etag updates mutex poisoned");
    etag_updates.extend(detail_etag_updates.clone());

    let detail_last_modified_updates = detail_last_modified_updates
        .lock()
        .expect("detail last-modified updates mutex poisoned");
    last_modified_updates.extend(detail_last_modified_updates.clone());
}

fn polite_delay_for_path(path: &str) -> Duration {
    let hash = path
        .as_bytes()
        .iter()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(*b as u64));
    let jitter = hash % DETAIL_FETCH_JITTER_MS.max(1);
    Duration::from_millis(DETAIL_FETCH_BASE_DELAY_MS + jitter)
}

#[derive(Clone, Debug)]
enum CachedFetch {
    NotModified,
    Modified {
        html: String,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

fn fetch_url_with_cache_reqwest(
    client: &Client,
    url: &str,
    etag: Option<&String>,
    last_modified: Option<&String>,
) -> Result<CachedFetch, reqwest::Error> {
    let mut request = client.get(url);
    if let Some(etag) = etag {
        request = request.header(IF_NONE_MATCH, etag);
    }
    if let Some(last_modified) = last_modified {
        request = request.header(IF_MODIFIED_SINCE, last_modified);
    }

    let response = request.send()?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(CachedFetch::NotModified);
    }

    let response = response.error_for_status()?;
    let new_etag = response
        .headers()
        .get(ETAG)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let new_last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let html = response.text()?;
    Ok(CachedFetch::Modified {
        html,
        etag: new_etag,
        last_modified: new_last_modified,
    })
}

fn resolve_wiki_url(path_or_url: &str) -> String {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        path_or_url.to_string()
    } else if path_or_url.starts_with('/') {
        format!("https://wiki.pokexgames.com{path_or_url}")
    } else {
        format!("https://wiki.pokexgames.com/{path_or_url}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CachedFetch, HttpCacheClient, scrape_source_incremental_with_http,
    };
    use super::super::WikiSource;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct FakeHttpClient {
        responses: HashMap<String, Result<CachedFetch, String>>,
    }

    impl HttpCacheClient for FakeHttpClient {
        fn fetch_url_with_cache(
            &self,
            url: &str,
            _etag: Option<&String>,
            _last_modified: Option<&String>,
        ) -> Result<CachedFetch, String> {
            self.responses
                .get(url)
                .cloned()
                .unwrap_or_else(|| Err(format!("missing fake response for {url}")))
        }
    }

    #[test]
    fn scrape_source_uses_existing_price_map_when_list_has_no_price() {
        let list_url = WikiSource::Loot.url().to_string();
        let html = r#"
            <table>
                <tr>
                    <td><a title="Tech Data" href="/wiki/tech_data">Tech Data</a></td>
                </tr>
            </table>
        "#;

        let mut responses = HashMap::new();
        responses.insert(
            list_url,
            Ok(CachedFetch::Modified {
                html: html.to_string(),
                etag: Some("etag-list".to_string()),
                last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
            }),
        );

        let http = Arc::new(FakeHttpClient { responses });
        let existing_price_map = HashMap::from([(String::from("tech data"), String::from("7k"))]);

        let data = scrape_source_incremental_with_http(
            http,
            WikiSource::Loot,
            &existing_price_map,
            &HashMap::new(),
            &HashMap::new(),
        )
        .expect("source scrape should succeed with fake list html");

        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].name, "Tech Data");
        assert_eq!(data.items[0].npc_price.as_deref(), Some("7k"));
        assert_eq!(data.items[0].sources, vec![WikiSource::Loot]);
        assert_eq!(data.etag_cache.get(WikiSource::Loot.url()).map(String::as_str), Some("etag-list"));
    }

    #[test]
    fn scrape_source_fetches_detail_price_when_missing_on_list() {
        let list_url = WikiSource::Loot.url().to_string();
        let detail_url = "https://wiki.pokexgames.com/wiki/ancient_wire".to_string();

        let list_html = r#"
            <table>
                <tr>
                    <td><a title="Ancient Wire" href="/wiki/ancient_wire">Ancient Wire</a></td>
                </tr>
            </table>
        "#;

        let detail_html = r#"
            <table>
                <tr><th>Preco NPC</th><td>12k</td></tr>
            </table>
        "#;

        let mut responses = HashMap::new();
        responses.insert(
            list_url,
            Ok(CachedFetch::Modified {
                html: list_html.to_string(),
                etag: None,
                last_modified: None,
            }),
        );
        responses.insert(
            detail_url.clone(),
            Ok(CachedFetch::Modified {
                html: detail_html.to_string(),
                etag: Some("etag-detail".to_string()),
                last_modified: Some("Thu, 22 Oct 2015 07:28:00 GMT".to_string()),
            }),
        );

        let http = Arc::new(FakeHttpClient { responses });
        let data = scrape_source_incremental_with_http(
            http,
            WikiSource::Loot,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .expect("source scrape should resolve missing detail price");

        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].name, "Ancient Wire");
        assert_eq!(data.items[0].npc_price.as_deref(), Some("12k"));
        assert_eq!(
            data.etag_cache.get(&detail_url).map(String::as_str),
            Some("etag-detail")
        );
    }
}
