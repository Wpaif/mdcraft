use crate::data::wiki_scraper::ScrapedCraftRecipe;

use super::normalize::normalized_ingredient_key;

pub(super) fn build_ingredient_vocabulary(recipes: &[ScrapedCraftRecipe]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut vocab = Vec::new();

    for recipe in recipes {
        for ingredient in &recipe.ingredients {
            if seen.insert(ingredient.name.clone()) {
                vocab.push(ingredient.name.clone());
            }
        }
    }

    vocab
}

pub(super) fn fuzzy_resolve_ingredient<'a>(raw: &'a str, vocab: &'a [String]) -> &'a str {
    let raw_key = normalized_ingredient_key(raw);

    for canonical in vocab {
        if normalized_ingredient_key(canonical) == raw_key {
            return canonical.as_str();
        }
    }

    const THRESHOLD: f64 = 0.80;
    let mut best_score = THRESHOLD;
    let mut best_match: Option<&str> = None;
    let mut ambiguous = false;

    for canonical in vocab {
        let key = normalized_ingredient_key(canonical);
        let score = strsim::normalized_levenshtein(&raw_key, &key);

        #[allow(clippy::float_cmp)]
        if score > best_score {
            best_score = score;
            best_match = Some(canonical.as_str());
            ambiguous = false;
        } else if score == best_score && best_match.is_some() {
            ambiguous = true;
        }
    }

    if ambiguous {
        raw
    } else {
        best_match.unwrap_or(raw)
    }
}
