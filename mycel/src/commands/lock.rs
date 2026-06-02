use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

const LOCKS_DIR: &str = "/var/lib/mycel/locks";

pub fn run(package: &str) -> Result<()> {
    fs::create_dir_all(LOCKS_DIR)
        .with_context(|| format!("could not create {}", LOCKS_DIR))?;

    let marker = format!("{}/{}", LOCKS_DIR, package);

    if std::path::Path::new(&marker).exists() {
        println!("{} {} is already locked", "::".dimmed(), package.bold());
        return Ok(());
    }

    fs::write(&marker, "")
        .with_context(|| format!("could not write lock marker for '{}'", package))?;

    println!("{} {} locked — will survive rollbacks and switch removals",
        "ok".green().bold(), package.bold());
    Ok(())
}

/// Returns all packages that have a lock marker on disk.
pub fn locked_packages() -> std::collections::HashSet<String> {
    fs::read_dir(LOCKS_DIR)
        .map(|entries| {
            entries.flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}
