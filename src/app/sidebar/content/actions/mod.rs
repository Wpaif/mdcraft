use crate::app::{MdcraftApp, apply_saved_item_prices};

mod recipe_quantity;
#[cfg(test)]
mod tests;

pub(super) fn apply_pending_sidebar_actions(
    app: &mut MdcraftApp,
    pending_click_delete: Option<usize>,
    pending_click_select: Option<usize>,
) {
    if let Some(idx) = pending_click_delete {
        app.pending_delete_index = Some(idx);
    }

    if let Some(idx) = pending_click_select {
        load_saved_craft_for_edit(app, idx);
    }
}

pub(super) fn set_pending_action(slot: &mut Option<usize>, idx: usize, clicked: bool) {
    if clicked {
        *slot = Some(idx);
    }
}

pub(super) fn load_saved_craft_for_edit(app: &mut MdcraftApp, idx: usize) {
    let Some(craft) = app.saved_crafts.get(idx) else {
        return;
    };

    app.sell_price_input = craft.sell_price_input.clone();
    app.sell_price_is_per_item = craft.sell_price_is_per_item;
    app.items.clear();

    // Reconstrói os itens a partir dos ingredientes do cache (preferencial).
    if let Some(recipe) = app
        .craft_recipes_cache
        .iter()
        .find(|r| r.name == craft.name)
    {
        let craft_count =
            recipe_quantity::infer_craft_count_from_saved_recipe(craft, recipe).unwrap_or(1);
        app.craft_search_qty = craft_count.max(1);
        app.craft_search_qty_input = app.craft_search_qty.to_string();

        for ing in &recipe.ingredients {
            let is_resource = app
                .resource_list
                .iter()
                .any(|res| res.eq_ignore_ascii_case(&ing.name));
            let base_qty = ing.quantity as u64;
            let mut item = crate::model::Item {
                nome: ing.name.clone(),
                quantidade: base_qty.saturating_mul(app.craft_search_qty),
                quantidade_base: base_qty,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource,
                preco_input: String::new(),
            };
            crate::app::ui_sections::craft_input::apply_cached_npc_price_if_available(
                app, &mut item,
            );
            app.items.push(item);
        }
    } else {
        // Fallback: se o craft não existir no cache atual, usa o texto salvo.
        // Mantém as quantidades exatas do texto para evitar cálculos errados.
        let qty_map = recipe_quantity::quantities_from_recipe_text(&craft.recipe_text);
        for (name_key, qty) in qty_map {
            let display_name = recipe_quantity::display_name_from_normalized_key(&name_key);
            let is_resource = app
                .resource_list
                .iter()
                .any(|res| res.eq_ignore_ascii_case(&display_name));
            let mut item = crate::model::Item {
                nome: display_name,
                quantidade: qty,
                quantidade_base: qty,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource,
                preco_input: String::new(),
            };
            crate::app::ui_sections::craft_input::apply_cached_npc_price_if_available(
                app, &mut item,
            );
            app.items.push(item);
        }
    }

    apply_saved_item_prices(&mut app.items, &craft.item_prices);
    app.active_saved_craft_index = Some(idx);
    app.selected_craft_name = craft.name.clone();
}

