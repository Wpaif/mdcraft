mod detail_prices;
mod resolve;

use reqwest::Client as AsyncClient;
use std::collections::HashMap;

use super::{items_parser, ScrapeError, ScrapeRefreshData, WikiSource};

pub async fn scrape_source_incremental_with_cache_async(
    client: &AsyncClient,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
    _etag_cache: &HashMap<String, String>,
    _last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let url = source.url();
    let html = client
        .get(url)
        .send()
        .await
        .map_err(|e| ScrapeError::Request {
            source,
            message: e.to_string(),
        })?
        .text()
        .await
        .map_err(|e| ScrapeError::Request {
            source,
            message: e.to_string(),
        })?;

    let mut rows = items_parser::parse_item_rows_from_html(&html, source);

    // Reaproveita preços do cache local antes de buscar detalhes.
    for row in &mut rows {
        if row.item.npc_price.is_some() {
            continue;
        }
        if let Some(price) = existing_price_map.get(&items_parser::normalize_key(&row.item.name)) {
            row.item.npc_price = Some(price.clone());
        }
    }

    detail_prices::fill_missing_prices_from_details_async(client, &mut rows, existing_price_map)
        .await;

    Ok(ScrapeRefreshData {
        items: rows.into_iter().map(|row| row.item).collect(),
        etag_cache: HashMap::new(),
        last_modified_cache: HashMap::new(),
    })
}

