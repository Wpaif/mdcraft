use std::collections::HashMap;

use reqwest::Client as AsyncClient;

use super::resolve::resolve_wiki_url;
use super::super::items_parser;

/// Preenche os preços NPC dos itens sem preço, buscando a página de detalhes de cada item em lotes.
pub(super) async fn fill_missing_prices_from_details_async(
    client: &AsyncClient,
    rows: &mut [items_parser::ParsedItemRow],
    existing_price_map: &HashMap<String, String>,
) {
    use futures::stream::{FuturesUnordered, StreamExt};
    use tokio::time::{sleep, Duration};

    fn format_item_list(rows: &[items_parser::ParsedItemRow], indices: &[usize]) -> String {
        let mut names = indices
            .iter()
            .filter_map(|&idx| rows.get(idx).map(|row| row.item.name.trim().to_string()))
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();

        match names.len() {
            0 => "<desconhecido>".to_string(),
            1..=3 => names.join(", "),
            n => {
                let shown = names.into_iter().take(3).collect::<Vec<_>>().join(", ");
                format!("{shown} (+{} outros)", n - 3)
            }
        }
    }

    // Primeiro: aplica preços conhecidos do cache local para reduzir chamadas HTTP.
    for row in rows.iter_mut() {
        if row.item.npc_price.is_some() {
            continue;
        }
        if let Some(price) = existing_price_map.get(&items_parser::normalize_key(&row.item.name)) {
            row.item.npc_price = Some(price.clone());
        }
    }

    // Agrupa por URL de detalhe (pode haver duplicatas).
    let mut url_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        if row.item.npc_price.is_some() {
            continue;
        }
        let Some(path) = row.detail_path.as_ref() else {
            continue;
        };
        let Some(url) = resolve_wiki_url(path) else {
            continue;
        };
        url_to_indices.entry(url).or_default().push(idx);
    }

    let urls_all = url_to_indices.keys().cloned().collect::<Vec<_>>();
    if urls_all.is_empty() {
        return;
    }

    // Parâmetros de lote
    let batch_size = 8;
    let delay_ms = 350;
    let mut idx = 0;
    let mut total_success = 0;
    let mut total_fail = 0;
    while idx < urls_all.len() {
        let end = (idx + batch_size).min(urls_all.len());
        let urls = &urls_all[idx..end];

        println!(
            "[async-scrape] Iniciando lote de {} requisições de detalhes...",
            urls.len()
        );

        let mut futures = FuturesUnordered::new();
        for url in urls {
            let client = client.clone();
            let url = url.clone();
            futures.push(async move {
                let resp = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("erro HTTP: {e}"))?;
                let html = resp
                    .text()
                    .await
                    .map_err(|e| format!("erro lendo body: {e}"))?;
                Ok::<(String, String), String>((url, html))
            });
        }

        let mut url_to_price: HashMap<String, String> = HashMap::new();
        let mut url_to_error: HashMap<String, String> = HashMap::new();
        let mut batch_success = 0;
        let mut batch_fail = 0;
        while let Some(res) = futures.next().await {
            match res {
                Ok((url, html)) => {
                    let mut price = items_parser::extract_npc_price_from_item_detail(&html);
                    if price.is_none() {
                        if let Some(href) = items_parser::extract_mediawiki_redirect_target_href(&html) {
                            if let Some(redirect_url) = resolve_wiki_url(&href) {
                                if redirect_url != url {
                                    if let Ok(resp) = client.get(&redirect_url).send().await {
                                        if let Ok(html2) = resp.text().await {
                                            price = items_parser::extract_npc_price_from_item_detail(&html2);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(price) = price {
                        url_to_price.insert(url, price);
                        batch_success += 1;
                    } else {
                        url_to_error.insert(url, "Preço NPC não encontrado".to_string());
                        batch_fail += 1;
                    }
                }
                Err(err) => {
                    batch_fail += 1;
                    eprintln!("[async-scrape] Falha ao buscar detalhe: {err}");
                }
            }
        }

        // Preencher nos rows (todas as linhas que apontam para a mesma URL).
        for url in urls {
            let Some(indices) = url_to_indices.get(url) else {
                continue;
            };

            if let Some(price) = url_to_price.get(url) {
                for &row_idx in indices {
                    rows[row_idx].item.npc_price = Some(price.clone());
                }
                println!(
                    "[async-scrape] OK: {} -> {}",
                    format_item_list(rows, indices),
                    price
                );
            } else {
                let reason = url_to_error
                    .get(url)
                    .map(String::as_str)
                    .unwrap_or("falha desconhecida");
                println!(
                    "[async-scrape] FALHOU: {} ({})",
                    format_item_list(rows, indices),
                    reason
                );
            }
        }

        total_success += batch_success;
        total_fail += batch_fail;
        println!(
            "[async-scrape] Lote finalizado: {} preços extraídos, {} falhas.",
            batch_success, batch_fail
        );

        idx = end;
        if idx < urls_all.len() {
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    println!(
        "[async-scrape] Resumo final: {} preços extraídos, {} falhas.",
        total_success, total_fail
    );
}
