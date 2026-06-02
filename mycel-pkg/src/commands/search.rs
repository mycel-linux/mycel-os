use anyhow::Result;
use colored::Colorize;

const INDEX_URL: &str = "https://raw.githubusercontent.com/mycel-linux/mycel-os/main/community/index.toml";

#[derive(serde::Deserialize)]
struct Index {
    overlays: Option<Vec<Overlay>>,
}

#[derive(serde::Deserialize)]
struct Overlay {
    name:        String,
    repo:        String,
    description: String,
    maintainer:  Option<String>,
}

pub fn run(query: &str) -> Result<()> {
    print!("{} fetching community index... ", "::".blue().bold());

    let response = reqwest::blocking::get(INDEX_URL);

    match response {
        Ok(r) => {
            println!("{}", "ok".green());
            let text = r.text()?;
            let index: Index = toml::from_str(&text)?;

            let overlays = index.overlays.unwrap_or_default();
            let results: Vec<&Overlay> = overlays.iter()
                .filter(|o| {
                    o.name.contains(query)
                        || o.description.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();

            if results.is_empty() {
                println!("{} no results for '{}'", "::".dimmed(), query);
            } else {
                println!();
                for overlay in results {
                    println!("  {} — {}", overlay.name.bold(), overlay.description);
                    println!("  {} {}", "add:".dimmed(), overlay.repo.dimmed());
                    println!();
                }
            }
        }
        Err(_) => {
            println!("{}", "offline".yellow());
            println!("{} could not reach community index", "!!".yellow().bold());
        }
    }

    Ok(())
}
