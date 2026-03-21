use std::time::{Duration, Instant};

use crate::app::MdcraftApp;

const RECIPE_AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(700);

impl MdcraftApp {
    pub(crate) fn schedule_active_recipe_autosave(&mut self) {
        if self.active_saved_craft_index.is_none() {
            return;
        }

        crate::app::ui_sections::autosave_active_craft(self);
        self.recipe_autosave_dirty = true;
        self.recipe_autosave_last_change_at = Some(Instant::now());
    }

    pub(crate) fn poll_recipe_autosave(&mut self) {
        if !self.recipe_autosave_dirty {
            return;
        }
        if self.active_saved_craft_index.is_none() {
            self.recipe_autosave_dirty = false;
            self.recipe_autosave_last_change_at = None;
            return;
        }

        let Some(changed_at) = self.recipe_autosave_last_change_at else {
            self.recipe_autosave_dirty = false;
            return;
        };

        if changed_at.elapsed() < RECIPE_AUTOSAVE_DEBOUNCE {
            return;
        }

        self.persist_saved_crafts_to_sqlite();
        self.recipe_autosave_dirty = false;
        self.recipe_autosave_last_change_at = None;
    }
}

