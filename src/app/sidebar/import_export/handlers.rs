use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft};

use super::super::capitalize_display_name;
use super::json_codec::{format_json_pretty, parse_imported_saved_crafts};
use super::state::{
    apply_export_popup_result, close_export_popup, close_import_popup, mark_export_copied,
    open_export_popup, open_import_popup,
};

pub(super) fn insert_imported_crafts(app: &mut MdcraftApp, crafts: Vec<SavedCraft>) -> usize {
    let mut imported = 0usize;

    for craft in crafts.into_iter().rev() {
        let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
        let name = if craft.name.trim().is_empty() {
            fallback_name
        } else {
            capitalize_display_name(&craft.name)
        };

        app.saved_crafts.insert(
            0,
            SavedCraft {
                name,
                recipe_text: craft.recipe_text,
                sell_price_input: craft.sell_price_input,
                item_prices: craft.item_prices,
            },
        );
        imported += 1;
    }

    if imported > 0 {
        app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + imported);
    }

    imported
}

pub(super) fn handle_sidebar_import_click(app: &mut MdcraftApp, import_clicked: bool) {
    if import_clicked {
        open_import_popup(app);
    }
}

pub(super) fn handle_sidebar_export_click(app: &mut MdcraftApp, export_clicked: bool) {
    if export_clicked {
        let result = open_export_popup(app);
        apply_export_popup_result(app, result);
    }
}

pub(super) fn handle_import_format_click(app: &mut MdcraftApp, format_clicked: bool) {
    if !format_clicked {
        return;
    }

    let raw_json = app.import_json_input.trim();
    if raw_json.is_empty() {
        app.import_feedback = Some("Cole um JSON antes de formatar.".to_string());
    } else {
        match format_json_pretty(raw_json) {
            Ok(pretty) => {
                app.import_json_input = pretty;
                app.import_feedback = Some("JSON formatado com sucesso.".to_string());
            }
            Err(err) => {
                app.import_feedback = Some(err);
            }
        }
    }
}

pub(super) fn handle_import_confirm_click(app: &mut MdcraftApp, import_clicked: bool) {
    if !import_clicked {
        return;
    }

    let raw_json = app.import_json_input.trim();
    if raw_json.is_empty() {
        app.import_feedback = Some("Cole um JSON antes de importar.".to_string());
        return;
    }

    match parse_imported_saved_crafts(raw_json) {
        Ok(crafts) => {
            if crafts.is_empty() {
                app.import_feedback = Some("Nenhuma receita encontrada no JSON.".to_string());
                return;
            }

            let imported = insert_imported_crafts(app, crafts);
            app.persist_saved_crafts_to_sqlite();

            app.import_feedback =
                Some(format!("{} receita(s) importada(s) com sucesso.", imported));
            app.awaiting_import_json = false;
            app.import_json_input.clear();
        }
        Err(err) => {
            app.import_feedback = Some(err);
        }
    }
}

pub(super) fn handle_export_copy_click(ctx: &egui::Context, app: &mut MdcraftApp, copied: bool) {
    if copied {
        ctx.copy_text(app.export_json_output.clone());
        mark_export_copied(app);
    }
}

pub(super) fn handle_import_cancel_click(app: &mut MdcraftApp, cancel_clicked: bool) {
    if cancel_clicked {
        close_import_popup(app);
    }
}

pub(super) fn handle_export_close_click(app: &mut MdcraftApp, close_clicked: bool) {
    if close_clicked {
        close_export_popup(app);
    }
}
