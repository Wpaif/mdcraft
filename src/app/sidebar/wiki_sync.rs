use crate::app::MdcraftApp;

mod apply;
mod craft_refresh_flow;
mod refresh_flow;
mod schedule;

pub(super) fn ensure_wiki_refresh_started(app: &mut MdcraftApp) {
    refresh_flow::ensure_wiki_refresh_started(app);
}

pub(super) fn poll_wiki_refresh_result(app: &mut MdcraftApp) {
    refresh_flow::poll_wiki_refresh_result(app);
}

pub(super) fn poll_craft_refresh_result(app: &mut MdcraftApp) {
    craft_refresh_flow::poll_craft_refresh_result(app);
}

#[cfg(test)]
mod tests;
