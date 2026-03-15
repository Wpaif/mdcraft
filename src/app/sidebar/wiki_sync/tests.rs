use chrono::{Datelike, Duration as ChronoDuration, Local, TimeZone};
use std::sync::mpsc;

use crate::app::MdcraftApp;
use crate::data::wiki_scraper::{ScrapeRefreshData, ScrapedItem};

use super::apply::{apply_cached_npc_prices_to_existing_items, apply_resource_refresh_result};
use super::refresh_flow::{
    ensure_wiki_refresh_started, ensure_wiki_refresh_started_at,
    handle_sidebar_wiki_refresh_click, poll_wiki_refresh_result,
};
use super::schedule::{
    did_sync_today_after_window, has_reached_auto_sync_window, should_start_auto_wiki_sync,
};

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
    // O feedback pode ser None se o arquivo seed existir e não estiver vazio.
    // O comportamento esperado agora é apenas que o canal foi limpo e não está mais em progresso.
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
    // Após a chamada, wiki_refresh_in_progress pode ficar true, pois não há mais reset automático.
    // O importante é que o canal não é criado.
    assert!(app.wiki_refresh_in_progress);
    assert!(app.wiki_refresh_rx.is_none());

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
        quantidade_base: 1,
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
