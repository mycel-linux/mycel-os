use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use serde::Deserialize;

use crate::config::parser;
use crate::resolver::PackageIndex;

const DB_PATH: &str = "/var/lib/mycel/packages";

#[derive(Deserialize)]
struct InstalledRecord {
    version: String,
}

#[derive(Deserialize)]
struct RecipeVersion {
    package: RecipePkg,
}

#[derive(Deserialize)]
struct RecipePkg {
    version: String,
}

pub fn run() -> Result<()> {
    let config  = parser::load().context("could not load /etc/mycel.toml")?;
    let channel = config.system.channel.as_deref().unwrap_or("stable");
    let sources  = config.overlays.as_ref().map(|o| o.sources.clone()).unwrap_or_default();
    let index   = PackageIndex::build(&sources, channel)?;

    let installed: Vec<(String, String)> = fs::read_dir(DB_PATH)
        .map(|entries| {
            entries.flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension()?.to_str()? != "toml" { return None; }
                    let name = path.file_stem()?.to_str()?.to_string();
                    let content = fs::read_to_string(&path).ok()?;
                    let record: InstalledRecord = toml::from_str(&content).ok()?;
                    Some((name, record.version))
                })
                .collect()
        })
        .unwrap_or_default();

    if installed.is_empty() {
        println!("{} no packages installed", "::".dimmed());
        return Ok(());
    }

    println!("{} checking for updates (channel: {})...", "::".blue().bold(), channel.bold());
    println!();

    let mut updates_found = 0;

    for (name, installed_ver) in &installed {
        if let Some(recipe_path) = index.find(name) {
            if let Ok(content) = fs::read_to_string(recipe_path) {
                if let Ok(recipe) = toml::from_str::<RecipeVersion>(&content) {
                    let latest = &recipe.package.version;
                    if latest != installed_ver {
                        println!("  {} {} {} → {}",
                            "↑".blue().bold(),
                            name.bold(),
                            installed_ver.dimmed(),
                            latest.green().bold()
                        );
                        updates_found += 1;
                    }
                }
            }
        }
    }

    if updates_found == 0 {
        println!("{} everything is up to date", "ok".green().bold());
    } else {
        println!();
        println!("{} {} update(s) available — run {} to apply",
            "->".blue(),
            updates_found.to_string().bold(),
            "mycel switch".bold()
        );
    }

    Ok(())
}
