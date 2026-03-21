mod logic;
mod ui;
#[cfg(test)]
mod tests;

pub(super) use logic::{infer_craft_name_from_items, start_save_recipe_prompt, update_current_recipe};
pub(super) use ui::render_save_name_prompt;
