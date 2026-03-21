use crate::app::MdcraftApp;

use super::apply::apply_resource_refresh_result;
use super::schedule::{now_unix_seconds, should_start_auto_wiki_sync};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

fn start_wiki_refresh_async(app: &mut MdcraftApp) {
    let (tx, rx) = mpsc::channel();
    app.wiki_refresh_rx = Some(rx);

    // Snapshot do estado necessário: evita capturar `&mut app` na task.
    let existing = app.wiki_cached_items.clone();
    let etag_cache = app.wiki_http_etag_cache.clone();
    let last_modified_cache = app.wiki_http_last_modified_cache.clone();

    // Em unit tests não queremos iniciar rede nem runtime em background.
    if cfg!(test) {
        let _ = tx; // evita warning de variável não usada no modo de teste
        let _ = existing;
        let _ = etag_cache;
        let _ = last_modified_cache;
        return;
    }

    // Preferimos usar o runtime atual do tokio (o app roda sob `#[tokio::main]`).
    // Se por algum motivo não houver runtime, fazemos fallback criando um runtime em uma thread.
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("mdcraft/wiki-sync/0.1")
                .build();

            let result = async {
                let client = client.map_err(|e| e.to_string())?;
                crate::data::wiki_scraper::scrape_all_sources_incremental_async(
                    &client,
                    &existing,
                    &etag_cache,
                    &last_modified_cache,
                )
                .await
                .map_err(|e| e.to_string())
            }
            .await;

            // Se o receiver foi dropado (ex: app fechou), só ignoramos.
            let _ = tx.send(result);
        });
        return;
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();

        let result = match rt {
            Ok(rt) => rt.block_on(async {
                let client = reqwest::Client::builder()
                    .user_agent("mdcraft/wiki-sync/0.1")
                    .build()
                    .map_err(|e| e.to_string())?;
                crate::data::wiki_scraper::scrape_all_sources_incremental_async(
                    &client,
                    &existing,
                    &etag_cache,
                    &last_modified_cache,
                )
                .await
                .map_err(|e| e.to_string())
            }),
            Err(e) => Err(e.to_string()),
        };

        let _ = tx.send(result);
    });
}

pub(super) fn handle_sidebar_wiki_refresh_click(app: &mut MdcraftApp, refresh_clicked: bool) {
    if !refresh_clicked {
        return;
    }

    if app.wiki_refresh_in_progress {
        return;
    }

    app.wiki_refresh_in_progress = true;
    app.wiki_sync_feedback = Some("Atualizando base de itens em segundo plano...".to_string());
    app.wiki_sync_error_anim_started_at = None;
    app.wiki_sync_success_anim_started_at = None;

    eprintln!("[mdcraft][wiki-sync] Iniciando sincronização em segundo plano...");
    start_wiki_refresh_async(app);
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

            // Checa se o arquivo foi atualizado e não está vazio
            let path = "src/data/wiki_items_seed.json";
            let show_error = match std::fs::read_to_string(path) {
                Ok(content) => {
                    let trimmed = content.trim();
                    trimmed.is_empty() || trimmed == "[]" || trimmed == "[\n]"
                }
                Err(_) => true,
            };
            if show_error {
                app.wiki_sync_feedback = Some("Sincronizacao da wiki foi interrompida antes de concluir.".to_string());
                app.wiki_sync_success_anim_started_at = None;
                app.wiki_sync_error_anim_started_at = Some(std::time::Instant::now());
            } else {
                app.wiki_sync_feedback = None;
                app.wiki_sync_success_anim_started_at = None;
                app.wiki_sync_error_anim_started_at = None;
            }
        }
    }
}
