use super::WikiSource;

#[derive(Debug)]
pub enum ScrapeError {
    Request { source: WikiSource, message: String },
    Channel { message: String },
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request { source, message } => {
                write!(f, "failed to fetch {:?} source: {}", source, message)
            }
            Self::Channel { message } => {
                write!(f, "parallel scrape channel error: {}", message)
            }
        }
    }
}

impl std::error::Error for ScrapeError {}
