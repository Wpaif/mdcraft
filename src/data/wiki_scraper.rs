use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum WikiSource {
    Loot,
    Nightmare,
    DimensionalZone,
}

impl WikiSource {
    pub fn url(self) -> &'static str {
        match self {
            Self::Loot => {
                "https://wiki.pokexgames.com/index.php?title=Itens_de_Loot&mobileaction=toggle_view_desktop"
            }
            Self::Nightmare => {
                "https://wiki.pokexgames.com/index.php/Nightmare_Itens#Itens_Comuns-0"
            }
            Self::DimensionalZone => "https://wiki.pokexgames.com/index.php/Dimensional_Zone_Itens",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScrapedItem {
    pub name: String,
    pub npc_price: Option<String>,
    pub sources: Vec<WikiSource>,
}

#[derive(Clone, Debug, Default)]
pub struct ScrapeRefreshData {
    pub items: Vec<ScrapedItem>,
    pub etag_cache: HashMap<String, String>,
    pub last_modified_cache: HashMap<String, String>,
}

fn has_npc_price(item: &ScrapedItem) -> bool {
    item.npc_price
        .as_deref()
        .map(|p| !p.trim().is_empty())
        .unwrap_or(false)
}

fn retain_items_with_npc_price(items: Vec<ScrapedItem>) -> Vec<ScrapedItem> {
    items.into_iter().filter(has_npc_price).collect()
}

const EMBEDDED_WIKI_ITEMS: &str = include_str!("wiki_items_seed.json");
const EMBEDDED_RESOURCE_NAMES: &str = include_str!("resource_names_seed.json");
const DETAIL_FETCH_WORKERS: usize = 3;
const DETAIL_FETCH_BASE_DELAY_MS: u64 = 220;
const DETAIL_FETCH_JITTER_MS: u64 = 180;

pub fn embedded_wiki_items() -> Vec<ScrapedItem> {
    let items = serde_json::from_str(EMBEDDED_WIKI_ITEMS).unwrap_or_default();
    retain_items_with_npc_price(items)
}

pub fn embedded_resource_names() -> Vec<String> {
    serde_json::from_str(EMBEDDED_RESOURCE_NAMES).unwrap_or_default()
}

pub fn merge_item_lists(existing: &[ScrapedItem], incoming: &[ScrapedItem]) -> Vec<ScrapedItem> {
    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    merge_items(&mut merged, existing.to_vec());
    merge_items(&mut merged, incoming.to_vec());

    let mut items: Vec<ScrapedItem> = retain_items_with_npc_price(merged.into_values().collect());
    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

pub fn normalized_resource_names(items: &[ScrapedItem]) -> Vec<String> {
    let mut names: Vec<String> = items
        .iter()
        .map(|item| item.name.trim().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect();

    names.sort();
    names.dedup();
    names
}

#[derive(Debug)]
pub enum ScrapeError {
    Request { source: WikiSource, message: String },
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request { source, message } => {
                write!(f, "failed to fetch {:?} source: {}", source, message)
            }
        }
    }
}

impl std::error::Error for ScrapeError {}

#[allow(dead_code)]
pub fn scrape_all_sources(client: &Client) -> Result<Vec<ScrapedItem>, ScrapeError> {
    scrape_all_sources_incremental(client, &[], &HashMap::new(), &HashMap::new())
        .map(|data| retain_items_with_npc_price(data.items))
}

pub fn scrape_all_sources_incremental(
    client: &Client,
    existing: &[ScrapedItem],
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let sources = [
        WikiSource::Loot,
        WikiSource::Nightmare,
        WikiSource::DimensionalZone,
    ];

    let existing_price_map: HashMap<String, String> = existing
        .iter()
        .filter_map(|item| {
            item.npc_price
                .as_ref()
                .map(|price| (normalize_key(&item.name), price.clone()))
        })
        .collect();

    let (tx, rx) = mpsc::channel();
    for source in sources {
        let tx = tx.clone();
        let client = client.clone();
        let existing_price_map = existing_price_map.clone();
        let etag_cache = etag_cache.clone();
        let last_modified_cache = last_modified_cache.clone();

        thread::spawn(move || {
            let result = scrape_source_incremental_with_cache(
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

    let mut merged: HashMap<String, ScrapedItem> = HashMap::new();
    let mut merged_etags = etag_cache.clone();
    let mut merged_last_modified = last_modified_cache.clone();
    for _ in 0..sources.len() {
        let source_data = rx.recv().map_err(|err| ScrapeError::Request {
            source: WikiSource::Loot,
            message: format!("parallel scrape channel error: {err}"),
        })??;
        merge_items(&mut merged, source_data.items);
        merged_etags.extend(source_data.etag_cache);
        merged_last_modified.extend(source_data.last_modified_cache);
    }

    let mut items: Vec<ScrapedItem> = merged.into_values().collect();
    items = retain_items_with_npc_price(items);
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ScrapeRefreshData {
        items,
        etag_cache: merged_etags,
        last_modified_cache: merged_last_modified,
    })
}

#[allow(dead_code)]
pub fn scrape_source(client: &Client, source: WikiSource) -> Result<Vec<ScrapedItem>, ScrapeError> {
    scrape_source_incremental_with_cache(
        client,
        source,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    )
    .map(|data| retain_items_with_npc_price(data.items))
}

#[allow(dead_code)]
pub fn scrape_source_incremental(
    client: &Client,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    scrape_source_incremental_with_cache(
        client,
        source,
        existing_price_map,
        &HashMap::new(),
        &HashMap::new(),
    )
}

fn scrape_source_incremental_with_cache(
    client: &Client,
    source: WikiSource,
    existing_price_map: &HashMap<String, String>,
    etag_cache: &HashMap<String, String>,
    last_modified_cache: &HashMap<String, String>,
) -> Result<ScrapeRefreshData, ScrapeError> {
    let mut etag_updates = HashMap::new();
    let mut last_modified_updates = HashMap::new();
    let source_url = source.url().to_string();

    let html = match fetch_url_with_cache(
        client,
        &source_url,
        etag_cache.get(&source_url),
        last_modified_cache.get(&source_url),
    )
    .map_err(|err| ScrapeError::Request {
        source,
        message: err.to_string(),
    })? {
        CachedFetch::NotModified => {
            return Ok(ScrapeRefreshData {
                items: Vec::new(),
                etag_cache: etag_updates,
                last_modified_cache: last_modified_updates,
            });
        }
        CachedFetch::Modified {
            html,
            etag,
            last_modified,
        } => {
            if let Some(etag) = etag {
                etag_updates.insert(source_url.clone(), etag);
            }
            if let Some(last_modified) = last_modified {
                last_modified_updates.insert(source_url.clone(), last_modified);
            }
            html
        }
    };

    let mut rows = parse_item_rows_from_html(&html, source);
    for row in &mut rows {
        if row.item.npc_price.is_some() {
            continue;
        }

        if let Some(price) = existing_price_map.get(&normalize_key(&row.item.name)) {
            row.item.npc_price = Some(price.clone());
        }
    }

    fill_missing_prices_from_details(
        client,
        &mut rows,
        existing_price_map,
        etag_cache,
        last_modified_cache,
        &mut etag_updates,
        &mut last_modified_updates,
    );

    Ok(ScrapeRefreshData {
        items: retain_items_with_npc_price(rows.into_iter().map(|row| row.item).collect()),
        etag_cache: etag_updates,
        last_modified_cache: last_modified_updates,
    })
}

fn fill_missing_prices_from_details(
    client: &Client,
    rows: &mut [ParsedItemRow],
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
        .filter(|row| !existing_price_map.contains_key(&normalize_key(&row.item.name)))
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
        let client = client.clone();

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

                let detail_fetch = fetch_url_with_cache(
                    &client,
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

                let Some(price) = extract_npc_price_from_item_detail(&detail_html) else {
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

enum CachedFetch {
    NotModified,
    Modified {
        html: String,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

fn fetch_url_with_cache(
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

fn merge_items(merged: &mut HashMap<String, ScrapedItem>, new_items: Vec<ScrapedItem>) {
    for item in new_items {
        let key = normalize_key(&item.name);

        if let Some(existing) = merged.get_mut(&key) {
            if existing.npc_price.is_none() && item.npc_price.is_some() {
                existing.npc_price = item.npc_price.clone();
            }
            for source in item.sources {
                if !existing.sources.contains(&source) {
                    existing.sources.push(source);
                }
            }
        } else {
            merged.insert(key, item);
        }
    }
}

fn normalize_key(name: &str) -> String {
    name.trim().to_lowercase()
}

fn parse_items_from_html(html: &str, source: WikiSource) -> Vec<ScrapedItem> {
    parse_item_rows_from_html(html, source)
        .into_iter()
        .map(|row| row.item)
        .collect()
}

struct ParsedItemRow {
    item: ScrapedItem,
    detail_path: Option<String>,
}

fn parse_item_rows_from_html(html: &str, source: WikiSource) -> Vec<ParsedItemRow> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("td").expect("cell selector should be valid");

    let mut result = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let linked_items: Vec<(String, Option<String>)> = cells
            .iter()
            .filter_map(|cell| extract_name_and_detail_path_from_links(*cell))
            .collect();

        if !linked_items.is_empty() {
            // Loot tables often place many items in a single row. Parse one item per cell.
            for (name, detail_path) in linked_items {
                let npc_price = if cells.len() <= 2 {
                    extract_price_from_row(&cells, &name)
                } else {
                    None
                };

                result.push(ParsedItemRow {
                    item: ScrapedItem {
                        name,
                        npc_price,
                        sources: vec![source],
                    },
                    detail_path,
                });
            }
            continue;
        }

        let Some((name, detail_path)) = extract_name_and_detail_path_from_row(&cells) else {
            continue;
        };

        let npc_price = extract_price_from_row(&cells, &name);
        result.push(ParsedItemRow {
            item: ScrapedItem {
                name,
                npc_price,
                sources: vec![source],
            },
            detail_path,
        });
    }

    result
}

fn extract_name_and_detail_path_from_row(
    cells: &[ElementRef<'_>],
) -> Option<(String, Option<String>)> {
    for cell in cells {
        if let Some(name_and_path) = extract_name_and_detail_path_from_links(*cell) {
            return Some(name_and_path);
        }
    }

    for cell in cells {
        let text = clean_cell_text(&cell.text().collect::<String>());
        if is_valid_item_name(&text) {
            return Some((text, None));
        }
    }

    None
}

fn extract_name_from_row(cells: &[ElementRef<'_>]) -> Option<String> {
    extract_name_and_detail_path_from_row(cells).map(|(name, _)| name)
}

fn extract_name_and_detail_path_from_links(
    cell: ElementRef<'_>,
) -> Option<(String, Option<String>)> {
    let link_selector = Selector::parse("a[title]").expect("link selector should be valid");

    for link in cell.select(&link_selector) {
        let Some(title) = link.value().attr("title") else {
            continue;
        };

        if title.starts_with("Arquivo:") || title.starts_with("File:") {
            continue;
        }

        let normalized = clean_cell_text(title);
        if is_valid_item_name(&normalized) {
            let detail_path = link.value().attr("href").map(ToString::to_string);
            return Some((normalized, detail_path));
        }
    }

    None
}

fn extract_price_from_row(cells: &[ElementRef<'_>], extracted_name: &str) -> Option<String> {
    for cell in cells {
        let text = clean_cell_text(&cell.text().collect::<String>());

        if text.eq_ignore_ascii_case(extracted_name) {
            continue;
        }

        if let Some(price) = first_price_token(&text) {
            return Some(price);
        }
    }

    None
}

fn first_price_token(text: &str) -> Option<String> {
    let mut best_with_suffix: Option<String> = None;
    let mut best_plain: Option<String> = None;

    for part in text.split_whitespace() {
        let cleaned = part
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != ',')
            .to_string();

        if cleaned.is_empty() {
            continue;
        }

        let lower = cleaned.to_lowercase();
        let valid = lower
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == 'k')
            && lower.chars().any(|c| c.is_ascii_digit());

        if valid {
            if lower.ends_with('k') {
                best_with_suffix = Some(lower);
                break;
            }

            if best_plain.is_none() {
                best_plain = Some(lower);
            }
        }
    }

    best_with_suffix
        .or(best_plain)
        .and_then(|token| normalize_npc_price_text(&token))
}

fn normalize_npc_price_text(raw: &str) -> Option<String> {
    let value = parse_npc_price_value(raw)?;
    Some(format_npc_price_value(value))
}

fn parse_npc_price_value(raw: &str) -> Option<f64> {
    let value = raw.trim().to_lowercase().replace(',', ".");
    if value.is_empty() {
        return None;
    }

    let parsed = if value.ends_with("kk") {
        let number = normalize_numeric_literal(value.trim_end_matches("kk"))
            .parse::<f64>()
            .ok()?;
        number * 1_000_000.0
    } else if value.ends_with('k') {
        let number = normalize_numeric_literal(value.trim_end_matches('k'))
            .parse::<f64>()
            .ok()?;
        number * 1_000.0
    } else {
        normalize_numeric_literal(&value).parse::<f64>().ok()?
    };

    if parsed.is_finite() && parsed >= 0.0 {
        Some(parsed)
    } else {
        None
    }
}

fn normalize_numeric_literal(raw: &str) -> String {
    let candidate = raw.trim();
    let dot_count = candidate.chars().filter(|&c| c == '.').count();

    if dot_count > 1 {
        return candidate.replace('.', "");
    }

    if let Some((left, right)) = candidate.split_once('.') {
        if !left.is_empty()
            && right.len() == 3
            && left.chars().all(|c| c.is_ascii_digit())
            && right.chars().all(|c| c.is_ascii_digit())
        {
            return format!("{left}{right}");
        }
    }

    candidate.to_string()
}

fn format_npc_price_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{}kk", format_compact_decimal(value / 1_000_000.0))
    } else if value >= 1_000.0 {
        format!("{}k", format_compact_decimal(value / 1_000.0))
    } else {
        format_compact_decimal(value)
    }
}

fn format_compact_decimal(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        let mut s = format!("{:.2}", value);
        while s.contains('.') && s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
        s
    }
}

fn extract_npc_price_from_item_detail(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tr").expect("row selector should be valid");
    let cell_selector = Selector::parse("th, td").expect("cell selector should be valid");

    for row in document.select(&row_selector) {
        let cells: Vec<ElementRef<'_>> = row.select(&cell_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let header_text = clean_cell_text(&cells[0].text().collect::<String>()).to_lowercase();
        if !header_text.contains("preço npc") && !header_text.contains("preco npc") {
            continue;
        }

        for cell in cells.iter().skip(1) {
            let value_text = clean_cell_text(&cell.text().collect::<String>());
            if let Some(token) = first_price_token(&value_text) {
                return Some(token);
            }
        }
    }

    None
}

fn clean_cell_text(text: &str) -> String {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let mut kept = Vec::new();

    for (idx, part) in parts.iter().enumerate() {
        if looks_like_media_filename(part) {
            continue;
        }

        let next_is_media = parts
            .get(idx + 1)
            .map(|next| looks_like_media_filename(next))
            .unwrap_or(false);

        // Handles names split before extension like: "Ancient Wire.png Ancient Wire".
        if next_is_media {
            continue;
        }

        kept.push(*part);
    }

    kept.join(" ").trim().to_string()
}

fn looks_like_media_filename(part: &str) -> bool {
    let lower = part.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
}

fn is_valid_item_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let lower = name.to_lowercase();
    if lower == "item" || lower == "itens" || lower == "nightmare world" {
        return false;
    }

    lower.chars().any(|c| c.is_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::{
        ScrapedItem, WikiSource, clean_cell_text, embedded_resource_names, embedded_wiki_items,
        extract_name_from_row, extract_npc_price_from_item_detail, first_price_token,
        format_npc_price_value, is_valid_item_name, merge_item_lists, merge_items,
        normalize_npc_price_text, normalized_resource_names, parse_items_from_html,
    };
    use scraper::{Html, Selector};
    use std::collections::HashMap;

    #[test]
    fn clean_cell_text_removes_media_tokens() {
        let cleaned = clean_cell_text("Ancient Wire.png Ancient Wire");
        assert_eq!(cleaned, "Ancient Wire");
    }

    #[test]
    fn first_price_token_extracts_k_or_plain_number() {
        assert_eq!(first_price_token("Preco: 12k"), Some("12k".to_string()));
        assert_eq!(
            first_price_token("npc 4500 coins"),
            Some("4.5k".to_string())
        );
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
                <tr>
                    <td><a href="/index.php/Dog_Ear" title="Dog Ear">Dog Ear</a></td>
                    <td><a href="/index.php/Small_Tail" title="Small Tail">Small Tail</a></td>
                </tr>
            </table>
        "#;

        let items = parse_items_from_html(html, WikiSource::Loot);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Dog Ear");
        assert_eq!(items[1].name, "Small Tail");
        assert!(items.iter().all(|item| item.npc_price.is_none()));
    }

    #[test]
    fn merge_items_deduplicates_and_merges_sources() {
        let mut merged = HashMap::new();

        merge_items(
            &mut merged,
            vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: None,
                sources: vec![WikiSource::Loot],
            }],
        );

        merge_items(
            &mut merged,
            vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: Some("12k".to_string()),
                sources: vec![WikiSource::Nightmare],
            }],
        );

        let item = merged
            .get("ancient wire")
            .expect("merged item should exist");
        assert_eq!(item.npc_price.as_deref(), Some("12k"));
        assert!(item.sources.contains(&WikiSource::Loot));
        assert!(item.sources.contains(&WikiSource::Nightmare));
    }

    #[test]
    fn normalized_resource_names_sorts_and_deduplicates() {
        let names = normalized_resource_names(&[
            ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: None,
                sources: vec![],
            },
            ScrapedItem {
                name: " ancient wire ".to_string(),
                npc_price: None,
                sources: vec![],
            },
            ScrapedItem {
                name: "Gear Nose".to_string(),
                npc_price: None,
                sources: vec![],
            },
        ]);

        assert_eq!(
            names,
            vec!["ancient wire".to_string(), "gear nose".to_string()]
        );
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
    fn embedded_wiki_items_loads_seed_data() {
        let items = embedded_wiki_items();
        assert!(!items.is_empty());
        assert!(items.iter().any(|item| item.npc_price.is_some()));
    }

    #[test]
    fn embedded_resource_names_loads_seed_data() {
        let names = embedded_resource_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"tech data".to_string()));
    }

    #[test]
    fn merge_item_lists_keeps_existing_and_updates_prices() {
        let existing = vec![ScrapedItem {
            name: "Tech Data".to_string(),
            npc_price: None,
            sources: vec![WikiSource::Loot],
        }];
        let incoming = vec![ScrapedItem {
            name: "Tech Data".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Nightmare],
        }];

        let merged = merge_item_lists(&existing, &incoming);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].npc_price.as_deref(), Some("1k"));
        assert!(merged[0].sources.contains(&WikiSource::Loot));
        assert!(merged[0].sources.contains(&WikiSource::Nightmare));
    }
}
