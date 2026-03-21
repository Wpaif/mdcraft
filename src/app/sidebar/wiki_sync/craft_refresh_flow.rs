use crate::app::MdcraftApp;

use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

fn start_craft_refresh_async(app: &mut MdcraftApp) {
    let (tx, rx) = mpsc::channel();
    app.craft_refresh_rx = Some(rx);

    // Em unit tests não queremos iniciar rede nem runtime em background.
    if cfg!(test) {
        let _ = tx;
        return;
    }

    const TIMEOUT: Duration = Duration::from_secs(60);

    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("mdcraft/craft-sync/0.1")
                .build();

            let result = async {
                let client = client.map_err(|e| e.to_string())?;
                let crafts =
                    tokio::time::timeout(TIMEOUT, crate::data::wiki_scraper::crafts::scrape_all_profession_crafts_async(&client))
                        .await
                        .map_err(|_| "timeout ao baixar crafts da wiki".to_string())?
                        .map_err(|e| e.to_string())?;
                Ok(crafts)
            }
            .await;

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
                    .user_agent("mdcraft/craft-sync/0.1")
                    .build()
                    .map_err(|e| e.to_string())?;
                tokio::time::timeout(TIMEOUT, crate::data::wiki_scraper::crafts::scrape_all_profession_crafts_async(&client))
                    .await
                    .map_err(|_| "timeout ao baixar crafts da wiki".to_string())?
                    .map_err(|e| e.to_string())
            }),
            Err(e) => Err(e.to_string()),
        };

        let _ = tx.send(result);
    });
}

pub(super) fn ensure_craft_refresh_started(app: &mut MdcraftApp) {
    if app.craft_refresh_in_progress {
        return;
    }

    app.craft_refresh_in_progress = true;
    start_craft_refresh_async(app);
}

pub(super) fn poll_craft_refresh_result(app: &mut MdcraftApp) {
    if !app.craft_refresh_in_progress {
        return;
    }

    let recv_state = app
        .craft_refresh_rx
        .as_ref()
        .map(|rx| rx.try_recv())
        .unwrap_or(Err(TryRecvError::Disconnected));

    match recv_state {
        Ok(result) => {
            app.craft_refresh_in_progress = false;
            app.craft_refresh_rx = None;

            match result {
                Ok(crafts) => {
                    app.craft_recipes_cache = crafts;
                    app.craft_recipe_name_by_signature =
                        crate::app::build_craft_recipe_name_index(&app.craft_recipes_cache);
                }
                Err(e) => {
                    // Evita sobrescrever feedback útil de itens; só anexa quando apropriado.
                    let msg = format!("Itens ok, mas crafts falharam: {e}");
                    match app.wiki_sync_feedback.as_deref() {
                        Some(existing) if existing.contains("sincronizada") => {
                            app.wiki_sync_feedback = Some(format!("{existing} ({msg})"));
                        }
                        Some(existing) if existing.contains("craft") => {}
                        _ => app.wiki_sync_feedback = Some(msg),
                    }
                }
            }
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.craft_refresh_in_progress = false;
            app.craft_refresh_rx = None;
        }
    }
}

