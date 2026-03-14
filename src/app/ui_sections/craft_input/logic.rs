use crate::app::MdcraftApp;
use crate::app::npc_price_rules::fixed_npc_price_input;
use crate::parse::parse_clipboard;
use crate::parse::parse_price_flag;

use super::super::autosave_active_craft;

pub(super) fn lookup_cached_npc_price_input(app: &MdcraftApp, item_name: &str) -> Option<String> {
    let normalized = item_name.trim().to_lowercase();
    app.wiki_cached_items
        .iter()
        .find(|entry| entry.name.trim().to_lowercase() == normalized)
        .and_then(|entry| entry.npc_price.clone())
        .or_else(|| fixed_npc_price_input(item_name).map(ToString::to_string))
}

pub(super) fn apply_cached_npc_price_if_available(app: &MdcraftApp, item: &mut crate::model::Item) {
    let Some(npc_input) = lookup_cached_npc_price_input(app, &item.nome) else {
        return;
    };

    if parse_price_flag(&npc_input).is_err() {
        return;
    }

    item.preco_input = npc_input;
    item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0.0);
    item.valor_total = item.preco_unitario * item.quantidade as f64;
}

pub(super) fn rebuild_items_from_input(app: &mut MdcraftApp) {
    let resources: Vec<&str> = app.resource_list.iter().map(AsRef::as_ref).collect();
    let old_items = std::mem::take(&mut app.items);
    let mut new_items = parse_clipboard(&app.input_text, &resources);

    for new_item in &mut new_items {
        if let Some(old_item) = old_items.iter().find(|o| o.nome == new_item.nome) {
            new_item.preco_input = old_item.preco_input.clone();
            new_item.preco_unitario = old_item.preco_unitario;
            new_item.valor_total = new_item.preco_unitario * new_item.quantidade as f64;
        } else {
            apply_cached_npc_price_if_available(app, new_item);
        }
    }

    app.items = new_items;
    autosave_active_craft(app);
}

pub(super) fn apply_input_change(app: &mut MdcraftApp, changed: bool) {
    if changed {
        rebuild_items_from_input(app);
    }
}
