pub(super) fn normalized_item_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub(super) fn normalized_ingredient_key(name: &str) -> String {
    let key = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase();

    // Normalize plural/singular so both forms map to a stable key.
    key.strip_suffix('s').map(str::to_owned).unwrap_or(key)
}
