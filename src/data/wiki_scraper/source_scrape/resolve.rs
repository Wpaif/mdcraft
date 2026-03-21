use reqwest::Url;

const WIKI_HOST: &str = "wiki.pokexgames.com";

pub(super) fn resolve_wiki_url(path_or_url: &str) -> Option<String> {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        let Ok(url) = Url::parse(path_or_url) else {
            return None;
        };

        if url.scheme() == "https" && url.host_str() == Some(WIKI_HOST) {
            Some(url.to_string())
        } else {
            None
        }
    } else if path_or_url.starts_with('/') {
        Some(format!("https://{WIKI_HOST}{path_or_url}"))
    } else {
        Some(format!("https://{WIKI_HOST}/{path_or_url}"))
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_wiki_url;

    #[test]
    fn resolve_wiki_url_rejects_external_hosts_and_non_https() {
        assert_eq!(
            resolve_wiki_url("https://wiki.pokexgames.com/wiki/ok").as_deref(),
            Some("https://wiki.pokexgames.com/wiki/ok")
        );
        assert_eq!(resolve_wiki_url("https://evil.example/pwn"), None);
        assert_eq!(resolve_wiki_url("http://wiki.pokexgames.com/wiki/ok"), None);
        assert_eq!(
            resolve_wiki_url("/index.php/Butterfly_Wing").as_deref(),
            Some("https://wiki.pokexgames.com/index.php/Butterfly_Wing")
        );
    }
}

