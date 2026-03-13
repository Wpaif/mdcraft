use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::app::MdcraftApp;
use crate::data::wiki_scraper::{
    ScrapeRefreshData, ScrapedItem, scrape_all_sources_incremental,
};

use super::apply::apply_resource_refresh_result;
use super::schedule::{now_unix_seconds, should_start_auto_wiki_sync};

pub(super) fn handle_sidebar_wiki_refresh_click(app: &mut MdcraftApp, refresh_clicked: bool) {
    if !refresh_clicked {
        return;
    }

    if app.wiki_refresh_in_progress {
        return;
    }

    app.wiki_refresh_in_progress = true;
    app.wiki_sync_feedback = Some("Atualizando base de itens em segundo plano...".to_string());

    let existing_cache = app.wiki_cached_items.clone();
    let existing_etags = app.wiki_http_etag_cache.clone();
    let existing_last_modified = app.wiki_http_last_modified_cache.clone();
    let (tx, rx) = mpsc::channel();
    app.wiki_refresh_rx = Some(rx);

    thread::spawn(move || {
        let result = refresh_resource_list_from_wiki(
            &existing_cache,
            &existing_etags,
            &existing_last_modified,
        );
        let _ = tx.send(result);
    });
}

pub(super) fn ensure_wiki_refresh_started(app: &mut MdcraftApp) {
    app.wiki_refresh_started_on_launch = true;

    if app.wiki_refresh_in_progress {
        return;
    }

    let Some(now) = now_unix_seconds() else {
        return;
    };

    ensure_wiki_refresh_started_at(app, now);
}

pub(super) fn ensure_wiki_refresh_started_at(app: &mut MdcraftApp, now_unix_seconds: u64) {
    if app.wiki_refresh_in_progress {
        return;
    }

    if should_start_auto_wiki_sync(app, now_unix_seconds) {
        handle_sidebar_wiki_refresh_click(app, true);
    }
}

pub(super) fn poll_wiki_refresh_result(app: &mut MdcraftApp) {
    if !app.wiki_refresh_in_progress {
        return;
    }

    let recv_state = app
        .wiki_refresh_rx
        .as_ref()
        .map(|rx| rx.try_recv())
        .unwrap_or(Err(TryRecvError::Disconnected));

    match recv_state {
        Ok(result) => {
            apply_resource_refresh_result(app, result);
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.wiki_refresh_in_progress = false;
            app.wiki_refresh_rx = None;
            app.wiki_sync_feedback =
                Some("Sincronizacao da wiki foi interrompida antes de concluir.".to_string());
        }
    }
}

fn refresh_resource_list_from_wiki(
    existing_cache: &[ScrapedItem],
    existing_etags: &std::collections::HashMap<String, String>,
    existing_last_modified: &std::collections::HashMap<String, String>,
) -> Result<ScrapeRefreshData, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("mdcraft-wiki-scraper/0.1")
        .build()
        .map_err(|err| format!("Falha ao criar cliente HTTP: {err}"))?;

    let data = scrape_all_sources_incremental(
        &client,
        existing_cache,
        existing_etags,
        existing_last_modified,
    )
    .map_err(|err| format!("Falha ao coletar itens do wiki: {err}"))?;

    if data.items.is_empty() && existing_cache.is_empty() {
        return Err("Nenhum item foi encontrado nas paginas do wiki.".to_string());
    }

    Ok(data)
}
