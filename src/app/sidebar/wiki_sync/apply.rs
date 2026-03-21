use std::time::Instant;

use crate::app::MdcraftApp;
use crate::app::npc_price_rules::fixed_npc_price_input;
use crate::data::wiki_scraper::{ScrapeRefreshData, merge_item_lists};
use crate::parse::parse_price_flag;

use super::schedule::now_unix_seconds;
use super::craft_refresh_flow::ensure_craft_refresh_started;

pub(super) fn apply_cached_npc_prices_to_existing_items(app: &mut MdcraftApp) {
    // Fallback: se a base principal estiver vazia, tenta carregar do seed estático
    if app.wiki_cached_items.is_empty() {
        if let Ok(static_data) = std::fs::read_to_string("src/data/wiki_items_seed_static.json") {
            if let Ok(static_items) = serde_json::from_str::<Vec<crate::data::wiki_scraper::ScrapedItem>>(&static_data) {
                app.wiki_cached_items = static_items;
                println!("[fallback] Carregado seed estático de preços NPC");
            }
        }
    }
    for item in &mut app.items {
        if !item.preco_input.trim().is_empty() {
            continue;
        }

        let cached = app
            .wiki_cached_items
            .iter()
            .find(|entry| entry.name.trim().eq_ignore_ascii_case(item.nome.trim()))
            .and_then(|entry| entry.npc_price.clone())
            .or_else(|| fixed_npc_price_input(&item.nome));

        let Some(cached) = cached else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(&cached) else {
            continue;
        };

        item.preco_input = cached;
        item.preco_unitario = parsed;
        item.valor_total = parsed * item.quantidade as f64;
    }
}

pub(super) fn apply_resource_refresh_result(
    app: &mut MdcraftApp,
    result: Result<ScrapeRefreshData, String>,
) {
    app.wiki_refresh_in_progress = false;
    app.wiki_refresh_rx = None;

    match result {
        Ok(data) => {
            let merged = merge_item_lists(&app.wiki_cached_items, &data.items);
            let total = merged.len();
            let updated_count = data.items.len();
            app.wiki_cached_items = merged;
            app.wiki_http_etag_cache = data.etag_cache;
            app.wiki_http_last_modified_cache = data.last_modified_cache;
            apply_cached_npc_prices_to_existing_items(app);
            app.wiki_sync_feedback = Some(format!(
                "Base de itens sincronizada ({} total, {} atualizados nesta rodada).",
                total, updated_count
            ));
            app.wiki_sync_success_anim_started_at = Some(Instant::now());
            app.wiki_sync_error_anim_started_at = None;
            app.wiki_last_sync_unix_seconds = now_unix_seconds();
            eprintln!(
                "[mdcraft][wiki-sync] Sincronização concluída ({} total, {} atualizados).",
                total, updated_count
            );

            // Dispara atualização de crafts em background (não bloqueia a UI).
            if !cfg!(test) {
                ensure_craft_refresh_started(app);
            }

            // Limpa mensagem de erro anterior, se houver
            if let Some(feedback) = &app.wiki_sync_feedback {
                if feedback.contains("interrompida antes de concluir") || feedback.contains("falha") {
                    app.wiki_sync_feedback = None;
                }
            }
        }
        Err(err) => {
            let has_cached_data = !app.wiki_cached_items.is_empty();
            eprintln!("[mdcraft][wiki-sync] Erro na sincronização: {err}");
            if has_cached_data {
                app.wiki_sync_feedback =
                    Some(format!("Não foi possível sincronizar agora; usando base local. Detalhes: {err}"));
            } else {
                app.wiki_sync_feedback = Some(format!("Falha ao sincronizar com a wiki. Detalhes: {err}"));
            }
            app.wiki_sync_success_anim_started_at = None;
            app.wiki_sync_error_anim_started_at = Some(Instant::now());
        }
    }
}
