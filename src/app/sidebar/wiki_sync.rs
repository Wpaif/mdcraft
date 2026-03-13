use chrono::{Datelike, Local, TimeZone, Timelike};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::app::fixed_npc_price_input;
use crate::app::MdcraftApp;
use crate::data::wiki_scraper::{
    ScrapeRefreshData, ScrapedItem, merge_item_lists, scrape_all_sources_incremental,
};
use crate::parse::parse_price_flag;

const AUTO_WIKI_SYNC_TRIGGER_HOUR: u32 = 7;
const AUTO_WIKI_SYNC_TRIGGER_MINUTE: u32 = 40;

pub(super) fn now_unix_seconds() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn local_timestamp_to_day_and_minute(unix_seconds: u64) -> Option<(i32, u32, u32)> {
    let date_time = Local.timestamp_opt(unix_seconds as i64, 0).single()?;
    let minute_of_day = date_time.hour() * 60 + date_time.minute();
    Some((date_time.year(), date_time.ordinal(), minute_of_day))
}

fn has_reached_auto_sync_window(unix_seconds: u64) -> bool {
    let Some((_, _, minute_of_day)) = local_timestamp_to_day_and_minute(unix_seconds) else {
        return false;
    };

    let trigger_minute = AUTO_WIKI_SYNC_TRIGGER_HOUR * 60 + AUTO_WIKI_SYNC_TRIGGER_MINUTE;
    minute_of_day >= trigger_minute
}

fn is_same_local_day(left_unix_seconds: u64, right_unix_seconds: u64) -> bool {
    let Some((left_year, left_ordinal, _)) = local_timestamp_to_day_and_minute(left_unix_seconds)
    else {
        return false;
    };
    let Some((right_year, right_ordinal, _)) =
        local_timestamp_to_day_and_minute(right_unix_seconds)
    else {
        return false;
    };

    left_year == right_year && left_ordinal == right_ordinal
}

fn did_sync_today_after_window(last_sync_unix_seconds: u64, now_unix_seconds: u64) -> bool {
    if !is_same_local_day(last_sync_unix_seconds, now_unix_seconds) {
        return false;
    }

    has_reached_auto_sync_window(last_sync_unix_seconds)
}

fn should_start_auto_wiki_sync(app: &MdcraftApp, now_unix_seconds: u64) -> bool {
    if !has_reached_auto_sync_window(now_unix_seconds) {
        return false;
    }

    let Some(last_sync) = app.wiki_last_sync_unix_seconds else {
        return true;
    };

    !did_sync_today_after_window(last_sync, now_unix_seconds)
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

fn ensure_wiki_refresh_started_at(app: &mut MdcraftApp, now_unix_seconds: u64) {
    if app.wiki_refresh_in_progress {
        return;
    }

    if should_start_auto_wiki_sync(app, now_unix_seconds) {
        handle_sidebar_wiki_refresh_click(app, true);
    }
}

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
                Some("Sincronização da wiki foi interrompida antes de concluir.".to_string());
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
        return Err("Nenhum item foi encontrado nas páginas do wiki.".to_string());
    }

    Ok(data)
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

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Duration as ChronoDuration, Local, TimeZone};
    use std::sync::mpsc;

    use super::{
        apply_cached_npc_prices_to_existing_items, apply_resource_refresh_result,
        did_sync_today_after_window, ensure_wiki_refresh_started, ensure_wiki_refresh_started_at,
        handle_sidebar_wiki_refresh_click, has_reached_auto_sync_window, poll_wiki_refresh_result,
        should_start_auto_wiki_sync,
    };
    use crate::app::MdcraftApp;
    use crate::data::wiki_scraper::{ScrapeRefreshData, ScrapedItem};

    #[test]
    fn apply_resource_refresh_result_updates_resources_and_feedback() {
        let mut app = MdcraftApp::default();
        let initial_resources = app.resource_list.clone();
        let initial_len = app.wiki_cached_items.len();
        apply_resource_refresh_result(
            &mut app,
            Ok(ScrapeRefreshData {
                items: vec![
                    ScrapedItem {
                        name: "Ancient Wire".to_string(),
                        npc_price: Some("12k".to_string()),
                        sources: vec![],
                    },
                    ScrapedItem {
                        name: "Gear Nose".to_string(),
                        npc_price: None,
                        sources: vec![],
                    },
                ],
                etag_cache: std::collections::HashMap::new(),
                last_modified_cache: std::collections::HashMap::new(),
            }),
        );

        assert_eq!(app.resource_list, initial_resources);
        assert!(app.wiki_cached_items.len() >= initial_len);
        assert!(
            app.wiki_sync_feedback
                .as_deref()
                .expect("feedback should be set")
                .contains("sincronizada")
        );
    }

    #[test]
    fn apply_resource_refresh_result_stores_error_feedback() {
        let mut app = MdcraftApp::default();
        apply_resource_refresh_result(&mut app, Err("falha".to_string()));
        assert_eq!(app.wiki_sync_feedback.as_deref(), Some("falha"));
    }

    #[test]
    fn handle_sidebar_wiki_refresh_click_noop_when_not_clicked() {
        let mut app = MdcraftApp::default();
        let before = app.resource_list.clone();
        handle_sidebar_wiki_refresh_click(&mut app, false);
        assert_eq!(app.resource_list, before);
        assert_eq!(app.wiki_sync_feedback, None);
    }

    #[test]
    fn poll_wiki_refresh_result_applies_received_data() {
        let mut app = MdcraftApp::default();
        let initial_resources = app.resource_list.clone();
        let initial_len = app.wiki_cached_items.len();
        let (tx, rx) = mpsc::channel();
        app.wiki_refresh_in_progress = true;
        app.wiki_refresh_rx = Some(rx);

        tx.send(Ok(ScrapeRefreshData {
            items: vec![ScrapedItem {
                name: "Ancient Wire".to_string(),
                npc_price: Some("12k".to_string()),
                sources: vec![],
            }],
            etag_cache: std::collections::HashMap::new(),
            last_modified_cache: std::collections::HashMap::new(),
        }))
        .expect("channel should accept message");

        poll_wiki_refresh_result(&mut app);

        assert!(!app.wiki_refresh_in_progress);
        assert!(app.wiki_refresh_rx.is_none());
        assert_eq!(app.resource_list, initial_resources);
        assert!(app.wiki_cached_items.len() >= initial_len);
    }

    #[test]
    fn poll_wiki_refresh_result_handles_disconnected_channel() {
        let mut app = MdcraftApp::default();
        let (tx, rx) = mpsc::channel::<Result<ScrapeRefreshData, String>>();
        app.wiki_refresh_in_progress = true;
        app.wiki_refresh_rx = Some(rx);

        drop(tx);

        poll_wiki_refresh_result(&mut app);

        assert!(!app.wiki_refresh_in_progress);
        assert!(app.wiki_refresh_rx.is_none());
        assert!(
            app.wiki_sync_feedback
                .as_deref()
                .expect("feedback should exist")
                .contains("interrompida")
        );
    }

    fn local_timestamp_at(hour: u32, minute: u32) -> u64 {
        let now = Local::now();
        Local
            .with_ymd_and_hms(now.year(), now.month(), now.day(), hour, minute, 0)
            .single()
            .expect("valid local datetime")
            .timestamp() as u64
    }

    fn timestamp_of_previous_local_day(hour: u32, minute: u32) -> u64 {
        let now = Local::now();
        let today = Local
            .with_ymd_and_hms(now.year(), now.month(), now.day(), hour, minute, 0)
            .single()
            .expect("valid local datetime");

        (today - ChronoDuration::days(1)).timestamp() as u64
    }

    #[test]
    fn auto_sync_window_only_after_0740() {
        let before = local_timestamp_at(7, 39);
        let at = local_timestamp_at(7, 40);
        let after = local_timestamp_at(8, 5);

        assert!(!has_reached_auto_sync_window(before));
        assert!(has_reached_auto_sync_window(at));
        assert!(has_reached_auto_sync_window(after));
    }

    #[test]
    fn should_start_auto_wiki_sync_runs_once_per_day_after_window() {
        let mut app = MdcraftApp::default();
        let now_after_window = local_timestamp_at(8, 0);

        assert!(should_start_auto_wiki_sync(&app, now_after_window));

        app.wiki_last_sync_unix_seconds = Some(now_after_window);
        assert!(!should_start_auto_wiki_sync(&app, now_after_window));

        app.wiki_last_sync_unix_seconds = Some(local_timestamp_at(6, 0));
        assert!(should_start_auto_wiki_sync(&app, now_after_window));

        app.wiki_last_sync_unix_seconds = Some(timestamp_of_previous_local_day(8, 0));
        assert!(should_start_auto_wiki_sync(&app, now_after_window));
    }

    #[test]
    fn should_start_auto_wiki_sync_blocks_before_window() {
        let mut app = MdcraftApp::default();
        let now_before_window = local_timestamp_at(6, 30);

        assert!(!should_start_auto_wiki_sync(&app, now_before_window));

        app.wiki_last_sync_unix_seconds = Some(timestamp_of_previous_local_day(9, 0));
        assert!(!should_start_auto_wiki_sync(&app, now_before_window));
    }

    #[test]
    fn did_sync_today_after_window_respects_time_threshold() {
        let now_after_window = local_timestamp_at(9, 0);
        let today_before_window = local_timestamp_at(7, 0);
        let today_after_window = local_timestamp_at(8, 0);

        assert!(!did_sync_today_after_window(
            today_before_window,
            now_after_window
        ));
        assert!(did_sync_today_after_window(
            today_after_window,
            now_after_window
        ));
    }

    #[test]
    fn ensure_wiki_refresh_started_respects_schedule_and_progress_state() {
        let mut app = MdcraftApp::default();
        let now_after_window = local_timestamp_at(8, 0);
        app.wiki_last_sync_unix_seconds = Some(now_after_window);

        ensure_wiki_refresh_started(&mut app);
        assert!(app.wiki_refresh_started_on_launch);
        assert!(!app.wiki_refresh_in_progress);
        assert!(app.wiki_refresh_rx.is_none());

        app.wiki_last_sync_unix_seconds = Some(timestamp_of_previous_local_day(9, 0));
        ensure_wiki_refresh_started_at(&mut app, now_after_window);
        assert!(app.wiki_refresh_in_progress);
        assert!(app.wiki_refresh_rx.is_some());

        let rx_ptr = app.wiki_refresh_rx.as_ref().map(|rx| rx as *const _);

        ensure_wiki_refresh_started_at(&mut app, now_after_window);
        let rx_ptr_after = app.wiki_refresh_rx.as_ref().map(|rx| rx as *const _);
        assert_eq!(rx_ptr, rx_ptr_after);
    }

    #[test]
    fn apply_cached_npc_prices_to_existing_items_fills_empty_recipe_inputs() {
        let mut app = MdcraftApp::default();
        app.items = vec![crate::model::Item {
            nome: "Ancient Wire".to_string(),
            quantidade: 3,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: false,
            preco_input: String::new(),
        }];
        app.wiki_cached_items = vec![ScrapedItem {
            name: "Ancient Wire".to_string(),
            npc_price: Some("2k".to_string()),
            sources: vec![],
        }];

        apply_cached_npc_prices_to_existing_items(&mut app);

        assert_eq!(app.items[0].preco_input, "2k");
        assert_eq!(app.items[0].preco_unitario, 2000.0);
        assert_eq!(app.items[0].valor_total, 6000.0);
    }
}
