pub(in crate::data::wiki_scraper) fn normalize_key(name: &str) -> String {
    name.trim().to_lowercase()
}

pub(in crate::data::wiki_scraper) fn clean_cell_text(text: &str) -> String {
    // Alguns trechos da wiki vêm com non‑breaking spaces (ex: `Preço NPC`, `66 dólares`).
    // Normaliza para espaços comuns antes de fazer split.
    let normalized = text
        .replace('\u{00A0}', " ")
        .replace('\u{202F}', " ")
        .replace('\u{2007}', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    let mut kept = Vec::new();

    for (idx, part) in parts.iter().enumerate() {
        if looks_like_media_filename(part) {
            continue;
        }

        let next_is_media = parts
            .get(idx + 1)
            .map(|next| looks_like_media_filename(next))
            .unwrap_or(false);

        // Handles names split before extension like: "Ancient Wire.png Ancient Wire".
        if next_is_media {
            continue;
        }

        kept.push(*part);
    }

    kept.join(" ").trim().to_string()
}

fn looks_like_media_filename(part: &str) -> bool {
    let lower = part.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".webp")
}

pub(in crate::data::wiki_scraper) fn is_valid_item_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let lower = name.to_lowercase();
    if lower == "item" || lower == "itens" || lower == "nightmare world" {
        return false;
    }

    lower.chars().any(|c| c.is_alphabetic())
}
