use scraper::{Html, Selector};

pub(in crate::data::wiki_scraper) fn extract_mediawiki_redirect_target_href(
    html: &str,
) -> Option<String> {
    let document = Html::parse_document(html);
    let redirect_msg_selector = Selector::parse("div.redirectMsg a").ok();
    let redirect_text_selector = Selector::parse("ul.redirectText a").ok();

    for selector in [redirect_msg_selector, redirect_text_selector].into_iter().flatten() {
        if let Some(link) = document.select(&selector).next() {
            if let Some(href) = link.value().attr("href") {
                let href = href.trim();
                if !href.is_empty() {
                    return Some(href.to_string());
                }
            }
        }
    }

    None
}
