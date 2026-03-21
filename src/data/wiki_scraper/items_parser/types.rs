use super::super::ScrapedItem;

pub(in crate::data::wiki_scraper) struct ParsedItemRow {
    pub(in crate::data::wiki_scraper) item: ScrapedItem,
    pub(in crate::data::wiki_scraper) detail_path: Option<String>,
}
