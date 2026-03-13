#[path = "sqlite/io.rs"]
mod io;
#[path = "sqlite/paths.rs"]
mod paths;
#[path = "sqlite/schema.rs"]
mod schema;
#[cfg(test)]
#[path = "sqlite/tests.rs"]
mod tests;

pub(super) fn load_saved_crafts_from_sqlite() -> Result<Vec<crate::app::SavedCraft>, String> {
	io::load_saved_crafts_from_sqlite()
}

pub(super) fn save_saved_crafts_to_sqlite(
	saved_crafts: &[crate::app::SavedCraft],
) -> Result<(), String> {
	io::save_saved_crafts_to_sqlite(saved_crafts)
}

#[cfg(test)]
pub(super) fn load_saved_crafts_from_path(
	db_path: &std::path::Path,
) -> Result<Vec<crate::app::SavedCraft>, String> {
	io::load_saved_crafts_from_path(db_path)
}

#[cfg(test)]
pub(super) fn save_saved_crafts_to_path(
	db_path: &std::path::Path,
	saved_crafts: &[crate::app::SavedCraft],
) -> Result<(), String> {
	io::save_saved_crafts_to_path(db_path, saved_crafts)
}
