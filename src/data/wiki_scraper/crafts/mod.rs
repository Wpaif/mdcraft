mod error;
mod parse;
mod scrape;

pub type CraftScrapeError = error::CraftScrapeError;

pub fn parse_profession_crafts_from_html(
    html: &str,
    profession: super::CraftProfession,
) -> Vec<super::ScrapedCraftRecipe> {
    parse::parse_profession_crafts_from_html(html, profession)
}

pub async fn scrape_profession_crafts_async(
    client: &reqwest::Client,
    profession: super::CraftProfession,
) -> Result<Vec<super::ScrapedCraftRecipe>, CraftScrapeError> {
    scrape::scrape_profession_crafts_async(client, profession).await
}

pub async fn scrape_all_profession_crafts_async(
    client: &reqwest::Client,
) -> Result<Vec<super::ScrapedCraftRecipe>, CraftScrapeError> {
    let mut all = Vec::new();
    for profession in super::ALL_CRAFT_PROFESSIONS {
        let recipes = scrape_profession_crafts_async(client, profession).await?;
        all.extend(recipes);
    }
    Ok(all)
}

#[cfg(test)]
mod tests;
