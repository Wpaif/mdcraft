#[path = "../data/wiki_scraper.rs"]
#[allow(dead_code)]
mod wiki_scraper;

fn main() {
    if let Err(err) = run() {
        eprintln!("erro ao atualizar seed da wiki: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("refresh_wiki_seed.rs: modo síncrono removido. Use a pipeline assíncrona!");
    std::process::exit(1);
}
