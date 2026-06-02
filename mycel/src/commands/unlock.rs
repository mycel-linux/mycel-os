use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

const LOCKS_DIR: &str = "/var/lib/mycel/locks";

pub fn run(package: &str) -> Result<()> {
    let marker = format!("{}/{}", LOCKS_DIR, package);

    if !std::path::Path::new(&marker).exists() {
        println!("{} {} is not locked", "::".dimmed(), package.bold());
        return Ok(());
    }

    fs::remove_file(&marker)
        .with_context(|| format!("could not remove lock marker for '{}'", package))?;

    println!("{} {} unlocked — will be managed normally by switch",
        "ok".green().bold(), package.bold());
    Ok(())
}
