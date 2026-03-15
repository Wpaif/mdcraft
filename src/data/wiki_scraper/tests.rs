#[test]
fn test_embedded_wiki_items_usa_seed_estatico() {
    use crate::data::wiki_scraper::{embedded_wiki_items};
    use std::fs;
    // Lê o seed estático diretamente do arquivo
    let seed_json = fs::read_to_string("src/data/wiki_items_seed.json").expect("Seed JSON deve existir");
    let seed_items: Vec<crate::data::wiki_scraper::ScrapedItem> = serde_json::from_str(&seed_json).expect("Seed JSON válido");
    let app_items = embedded_wiki_items();
    // O seed embutido deve ser igual ao arquivo seed
    assert_eq!(app_items.len(), seed_items.len(), "Quantidade de itens do seed embutido difere do arquivo seed");
    // Checa alguns campos de exemplo
    assert_eq!(app_items[0].name, seed_items[0].name);
}

#[test]
fn test_seed_estatico_nao_muda_sem_atualizacao() {
    use crate::data::wiki_scraper::{embedded_wiki_items};
    let antes = embedded_wiki_items();
    // Simula que nenhuma atualização foi feita
    let depois = embedded_wiki_items();
    assert_eq!(antes, depois, "Seed embutido mudou sem atualização!");
}
//! Testes de integração reais com a wiki PokeXGames para scraping de itens e crafts.

#[tokio::test]
async fn test_scrape_items_from_wiki() {
    use crate::data::wiki_scraper::{scrape_all_sources_incremental_async, ScrapedItem, embedded_wiki_items};
    use reqwest::Client;
    use std::collections::HashMap;

    let client = Client::new();
    let existing: Vec<ScrapedItem> = vec![];
    let etag_cache = HashMap::new();
    let last_modified_cache = HashMap::new();

    let result = scrape_all_sources_incremental_async(&client, &existing, &etag_cache, &last_modified_cache).await;
    assert!(result.is_ok(), "Scraping falhou: {:?}", result);
    let data = result.unwrap();
    assert!(!data.items.is_empty(), "Nenhum item foi extraído da wiki!");
    let algum_com_preco = data.items.iter().any(|i| i.npc_price.is_some());
    assert!(algum_com_preco, "Nenhum item extraído possui preço NPC!");
}

#[tokio::test]
async fn test_scrape_crafts_from_wiki() {
    use crate::data::wiki_scraper::crafts::scrape_all_profession_crafts_async;
    use reqwest::Client;

    let client = Client::new();
    let crafts = scrape_all_profession_crafts_async(&client).await;
    assert!(crafts.is_ok(), "Scraping de crafts falhou: {:?}", crafts);
    let crafts = crafts.unwrap();
    assert!(!crafts.is_empty(), "Nenhum craft foi extraído da wiki!");
    let algum_com_nome = crafts.iter().any(|c| !c.name.trim().is_empty());
    assert!(algum_com_nome, "Nenhum craft extraído possui nome!");
}
