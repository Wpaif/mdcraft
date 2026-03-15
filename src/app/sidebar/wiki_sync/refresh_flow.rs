use crate::app::MdcraftApp;
// ...existing code...

use super::apply::apply_resource_refresh_result;
use super::schedule::{now_unix_seconds, should_start_auto_wiki_sync};
use std::sync::mpsc::TryRecvError;

pub(super) fn handle_sidebar_wiki_refresh_click(app: &mut MdcraftApp, refresh_clicked: bool) {
    if !refresh_clicked {
        return;
    }

    if app.wiki_refresh_in_progress {
        return;
    }

    app.wiki_refresh_in_progress = true;
    app.wiki_sync_feedback = Some("Atualizando base de itens em segundo plano...".to_string());

    // Pipeline síncrona removida. Use apenas a assíncrona!
    app.wiki_sync_feedback = Some("Sincronização síncrona removida. Use a versão assíncrona!".to_string());
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

