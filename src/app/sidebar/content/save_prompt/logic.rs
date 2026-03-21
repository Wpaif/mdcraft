use crate::app::state::RecipeSavePopupType;
use crate::app::{MdcraftApp, SavedCraft, capture_saved_item_prices};

use crate::app::capitalize_display_name;

pub(in crate::app::sidebar::content) fn infer_craft_name_from_items(
    app: &MdcraftApp,
) -> Option<String> {
    crate::app::infer_craft_name_from_items(
        &app.items,
        &app.craft_recipes_cache,
        &app.craft_recipe_name_by_signature,
    )
    .map(|name| capitalize_display_name(&name))
}

pub(in crate::app::sidebar::content) fn start_save_recipe_prompt(
    app: &mut MdcraftApp,
    save_clicked: bool,
) {
    if save_clicked {
        app.awaiting_craft_name = true;
        app.pending_craft_name = infer_craft_name_from_items(app).unwrap_or_default();
        app.focus_craft_name_input = true;
    }
}

pub(in crate::app::sidebar::content) fn update_current_recipe(app: &mut MdcraftApp) {
    let Some(idx) = app.active_saved_craft_index else {
        return;
    };
    if let Some(craft) = app.saved_crafts.get_mut(idx) {
        craft.recipe_text = app
            .items
            .iter()
            .map(|item| format!("{} {}", item.quantidade, capitalize_display_name(&item.nome)))
            .collect::<Vec<_>>()
            .join(", ");
        craft.sell_price_input = app.sell_price_input.clone();
        craft.item_prices = capture_saved_item_prices(&app.items);
    }
    app.persist_saved_crafts_to_sqlite();
    app.last_saved_recipe_name = Some(app.saved_crafts[idx].name.clone());
    app.recipe_save_toast_started_at = Some(std::time::Instant::now());
    app.show_recipe_save_popup = Some(RecipeSavePopupType::Update);
}

pub(super) fn commit_new_saved_craft(app: &mut MdcraftApp) {
    let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
    let raw_name = if app.pending_craft_name.trim().is_empty() {
        fallback_name
    } else {
        app.pending_craft_name.clone()
    };
    let normalized_name = capitalize_display_name(&raw_name);
    app.saved_crafts.insert(
        0,
        SavedCraft {
            name: normalized_name.clone(),
            recipe_text: String::new(),
            sell_price_input: app.sell_price_input.clone(),
            sell_price_is_per_item: app.sell_price_is_per_item,
            item_prices: capture_saved_item_prices(&app.items),
        },
    );
    app.active_saved_craft_index = Some(0);
    app.persist_saved_crafts_to_sqlite();
    app.awaiting_craft_name = false;
    app.pending_craft_name.clear();
    app.focus_craft_name_input = false;
    app.last_saved_recipe_name = Some(normalized_name);
    app.recipe_save_toast_started_at = Some(std::time::Instant::now());
    app.show_recipe_save_popup = Some(RecipeSavePopupType::Save);
}
