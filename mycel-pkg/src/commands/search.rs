use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::package::parser;

const CACHE_DIR: &str = "/var/lib/mycel/overlay-cache";

pub fn run(query: &str) -> Result<()> {
    let query_lower = query.to_lowercase();

    // Collect all .myc recipes from every cached overlay
    let mut results: Vec<(String, String, String)> = vec![];  // (name, version, description)

    let mut searched_overlays = 0usize;

    if let Ok(overlays) = fs::read_dir(CACHE_DIR) {
        for overlay in overlays.flatten() {
            if !overlay.path().is_dir() { continue; }
            searched_overlays += 1;
            scan_dir(&overlay.path(), &query_lower, &mut results);
        }
    }

    if searched_overlays == 0 {
        println!("{} no overlay cache found", "!!".yellow().bold());
        println!("  run {} first to populate the cache", "mycel update".bold());
        return Ok(());
    }

    // Deduplicate by name (keep first seen)
    let mut seen: HashMap<String, bool> = HashMap::new();
    results.retain(|(name, _, _)| seen.insert(name.clone(), true).is_none());
    results.sort_by(|a, b| a.0.cmp(&b.0));

    if results.is_empty() {
        println!("{} no packages found matching '{}'", "::".dimmed(), query);
        println!("  searched {} overlay(s)", searched_overlays);
    } else {
        println!("{} {} result(s) for '{}'",
            "::".blue().bold(),
            results.len().to_string().bold(),
            query.bold());
        println!("{}", "─────────────────────────────────────────".dimmed());
        println!();

        for (name, version, desc) in &results {
            println!("  {} {}",   name.bold(), version.dimmed());
            println!("  {}", desc);
            println!("  {} mycel get {}", "→".blue().dimmed(), name.dimmed());
            println!();
        }
    }

    Ok(())
}

fn scan_dir(dir: &Path, query: &str, results: &mut Vec<(String, String, String)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, query, results);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("myc") {
            continue;
        }
        if let Ok(recipe) = parser::load(path.to_str().unwrap_or("")) {
            let name  = recipe.package.name.to_lowercase();
            let desc  = recipe.package.description.to_lowercase();
            if name.contains(query) || desc.contains(query) {
                results.push((
                    recipe.package.name,
                    recipe.package.version,
                    recipe.package.description,
                ));
            }
        }
    }
}
