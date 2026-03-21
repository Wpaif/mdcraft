use super::error::CraftScrapeError;
use super::parse::parse_profession_crafts_from_html;
use super::super::{CraftProfession, ScrapedCraftRecipe};

fn parse_profession_crafts_from_html_result(
    html_result: Result<String, String>,
    profession: CraftProfession,
) -> Result<Vec<ScrapedCraftRecipe>, CraftScrapeError> {
    let html = html_result.map_err(|message| CraftScrapeError::Request { profession, message })?;
    Ok(parse_profession_crafts_from_html(&html, profession))
}

pub(super) async fn scrape_profession_crafts_async(
    client: &reqwest::Client,
    profession: CraftProfession,
) -> Result<Vec<ScrapedCraftRecipe>, CraftScrapeError> {
    let html_result = async {
        let resp = client
            .get(profession.url())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("HTTP status: {}", status));
        }
        resp.text().await.map_err(|e| e.to_string())
    }
    .await;
    parse_profession_crafts_from_html_result(html_result, profession)
}
