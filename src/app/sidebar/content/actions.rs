use crate::app::{MdcraftApp, apply_saved_item_prices};

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
    app.items.clear();
    // Reconstrói os itens a partir dos ingredientes da receita salva
    if let Some(recipe) = app.craft_recipes_cache.iter().find(|r| r.name == craft.name) {
        for ing in &recipe.ingredients {
            let is_resource = app.resource_list.iter().any(|res| res.eq_ignore_ascii_case(&ing.name));
            let mut item = crate::model::Item {
                nome: ing.name.clone(),
                quantidade: ing.quantity as u64,
                quantidade_base: ing.quantity as u64,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource,
                preco_input: String::new(),
            };
            crate::app::ui_sections::craft_input::apply_cached_npc_price_if_available(app, &mut item);
            app.items.push(item);
        }
    }
    apply_saved_item_prices(&mut app.items, &craft.item_prices);
    app.active_saved_craft_index = Some(idx);
    app.selected_craft_name = craft.name.clone();
}

#[cfg(test)]
mod tests {
    use crate::app::{MdcraftApp, SavedCraft, SavedItemPrice};

    use super::{apply_pending_sidebar_actions, load_saved_craft_for_edit, set_pending_action};

    fn make_saved_craft(name: &str, recipe_text: &str, sell_price_input: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: recipe_text.to_string(),
            sell_price_input: sell_price_input.to_string(),
            item_prices: vec![],
        }
    }

    #[test]
    fn load_saved_craft_for_edit_updates_active_data() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "teste".to_string(),
            recipe_text: "2 Iron Ore, 3 Screw".to_string(),
            sell_price_input: "12k".to_string(),
            item_prices: vec![SavedItemPrice {
                item_name: "Screw".to_string(),
                price_input: "250".to_string(),
            }],
        });

        load_saved_craft_for_edit(&mut app, 0);

        assert_eq!(app.active_saved_craft_index, Some(0));
        assert_eq!(app.sell_price_input, "12k");
        assert!(!app.items.is_empty());
        let screw = app
            .items
            .iter()
            .find(|i| i.nome == "Screw")
            .expect("Screw should exist after loading saved craft");
        assert_eq!(screw.preco_input, "250");
        assert_eq!(screw.preco_unitario, 250.0);
    }

    #[test]
    fn load_saved_craft_for_edit_ignores_out_of_bounds_index() {
        let mut app = MdcraftApp::default();
        app.saved_crafts
            .push(make_saved_craft("teste", "1 Iron Ore", "1k"));

        load_saved_craft_for_edit(&mut app, 9);

        assert_eq!(app.active_saved_craft_index, None);
    }

    #[test]
    fn apply_pending_sidebar_actions_sets_delete_and_selects_recipe() {
        let mut app = MdcraftApp::default();
        app.saved_crafts
            .push(make_saved_craft("receita a", "1 Iron Ore", "3k"));

        apply_pending_sidebar_actions(&mut app, Some(0), Some(0));

        assert_eq!(app.pending_delete_index, Some(0));
        assert_eq!(app.active_saved_craft_index, Some(0));
    }

    #[test]
    fn set_pending_action_respects_clicked_flag() {
        let mut slot = None;

        set_pending_action(&mut slot, 1, false);
        assert_eq!(slot, None);

        set_pending_action(&mut slot, 3, true);
        assert_eq!(slot, Some(3));
    }
}
