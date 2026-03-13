use crate::app::MdcraftApp;

pub(super) fn apply_delete_recipe(app: &mut MdcraftApp, idx: usize) {
    app.saved_crafts.remove(idx);

    if let Some(active_idx) = app.active_saved_craft_index {
        app.active_saved_craft_index = if active_idx == idx {
            None
        } else if active_idx > idx {
            Some(active_idx - 1)
        } else {
            Some(active_idx)
        };
    }

    app.persist_saved_crafts_to_sqlite();
    app.pending_delete_index = None;
}

pub(super) fn handle_cancel_delete_click(app: &mut MdcraftApp, clicked: bool) {
    if clicked {
        app.pending_delete_index = None;
    }
}

pub(super) fn handle_confirm_delete_click(app: &mut MdcraftApp, idx: usize, clicked: bool) {
    if clicked {
        apply_delete_recipe(app, idx);
    }
}
