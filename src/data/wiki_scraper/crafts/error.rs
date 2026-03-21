use super::super::CraftProfession;

#[derive(Debug)]
pub enum CraftScrapeError {
    Request {
        profession: CraftProfession,
        message: String,
    },
}

impl std::fmt::Display for CraftScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request {
                profession,
                message,
            } => {
                write!(
                    f,
                    "failed to fetch craft page for {:?}: {}",
                    profession, message
                )
            }
        }
    }
}

impl std::error::Error for CraftScrapeError {}

