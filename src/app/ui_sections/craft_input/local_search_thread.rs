// Thread de busca local para sugestões de receitas e ingredientes
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use crate::data::wiki_scraper::embedded_craft_recipes;

/// Mensagem para thread de busca
pub enum LocalSearchMsg {
    Query(String),
}

/// Resposta da thread de busca
pub enum LocalSearchResult {
    Suggestions(Vec<String>),
}

/// Inicia uma thread para buscar sugestões localmente e retorna Sender/Receiver
pub fn start_local_search_thread() -> (Sender<LocalSearchMsg>, Receiver<LocalSearchResult>) {
    let (tx, rx) = channel();
    let (result_tx, result_rx) = channel();
    thread::spawn(move || {
        // Busca local, sem ElasticSearch
        for msg in rx {
            match msg {
                LocalSearchMsg::Query(query) => {
                    let query_lower = query.to_lowercase();
                    // Busca receitas pelo nome do produto OU pelo nome de qualquer ingrediente
                    let recipes = embedded_craft_recipes();
                    use std::collections::HashSet;
                    let mut seen = HashSet::new();
                    let mut results: Vec<String> = recipes
                        .iter()
                        .filter(|recipe| {
                            recipe.name.to_lowercase().contains(&query_lower)
                                || recipe.ingredients.iter().any(|ing| ing.name.to_lowercase().contains(&query_lower))
                        })
                        .map(|recipe| recipe.name.clone())
                        .filter(|name| seen.insert(name.clone())) // deduplica pelo nome
                        .take(10)
                        .collect();
                    // Se nada encontrado, tenta fuzzy por similaridade no nome do produto ou ingredientes
                    if results.is_empty() && query_lower.len() > 2 {
                        let mut seen = HashSet::new();
                        results = recipes
                            .iter()
                            .filter(|recipe| {
                                strsim::levenshtein(&recipe.name.to_lowercase(), &query_lower) <= 2
                                    || recipe.ingredients.iter().any(|ing| strsim::levenshtein(&ing.name.to_lowercase(), &query_lower) <= 2)
                            })
                            .map(|recipe| recipe.name.clone())
                            .filter(|name| seen.insert(name.clone())) // deduplica pelo nome
                            .take(10)
                            .collect();
                    }
                    let _ = result_tx.send(LocalSearchResult::Suggestions(results));
                }
            }
        }
    });
    (tx, result_rx)
}
