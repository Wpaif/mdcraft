use crate::app::MdcraftApp;

use super::json_codec::build_export_json;

pub(super) fn open_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = true;
    app.import_feedback = None;
}

pub(super) fn close_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = false;
    app.import_feedback = None;
}

pub(super) fn open_export_popup(app: &mut MdcraftApp) -> Result<(), String> {
    let json = build_export_json(&app.saved_crafts)?;
    app.export_json_output = json;
    app.export_feedback = None;
    app.awaiting_export_json = true;
    Ok(())
}

pub(super) fn close_export_popup(app: &mut MdcraftApp) {
    app.awaiting_export_json = false;
    app.export_feedback = None;
}

pub(super) fn mark_export_copied(app: &mut MdcraftApp) {
    app.export_feedback = Some("JSON copiado para a area de transferencia.".to_string());
}

pub(super) fn apply_export_popup_result(app: &mut MdcraftApp, result: Result<(), String>) {
    if let Err(err) = result {
        app.export_feedback = Some(err);
        app.awaiting_export_json = true;
    }
}
