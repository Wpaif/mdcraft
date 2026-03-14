use crate::app::MdcraftApp;

mod apply;
mod refresh_flow;
mod schedule;

pub(super) fn handle_sidebar_wiki_refresh_click(app: &mut MdcraftApp, refresh_clicked: bool) {
    refresh_flow::handle_sidebar_wiki_refresh_click(app, refresh_clicked);
}

pub(super) fn ensure_wiki_refresh_started(app: &mut MdcraftApp) {
    refresh_flow::ensure_wiki_refresh_started(app);
}

pub(super) fn poll_wiki_refresh_result(app: &mut MdcraftApp) {
    refresh_flow::poll_wiki_refresh_result(app);
}

#[cfg(test)]
mod tests;
