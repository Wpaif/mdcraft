use std::time::Instant;

use crate::app::MdcraftApp;
use crate::app::npc_price_rules::fixed_npc_price_input;
use crate::data::wiki_scraper::{ScrapeRefreshData, merge_item_lists};
use crate::parse::parse_price_flag;

use super::schedule::now_unix_seconds;

pub(super) fn apply_cached_npc_prices_to_existing_items(app: &mut MdcraftApp) {
    for item in &mut app.items {
        if !item.preco_input.trim().is_empty() {
            continue;
        }

        let cached = app
            .wiki_cached_items
            .iter()
            .find(|entry| entry.name.trim().eq_ignore_ascii_case(item.nome.trim()))
            .and_then(|entry| entry.npc_price.as_deref())
            .or_else(|| fixed_npc_price_input(&item.nome));

        let Some(cached) = cached else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(cached) else {
            continue;
        };

        item.preco_input = cached.to_string();
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
            app.wiki_last_sync_unix_seconds = now_unix_seconds();
        }
        Err(err) => {
            app.wiki_sync_feedback = Some(err);
            app.wiki_sync_success_anim_started_at = None;
        }
    }
}
