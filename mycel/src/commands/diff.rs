use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::system::limine;

const DB_SUFFIX:   &str = "var/lib/mycel/packages";
const SNAPS_DIR:   &str = "/.snapshots";

pub fn run(gen1: &str, gen2: &str) -> Result<()> {
    let a: u64 = gen1.parse()
        .with_context(|| format!("'{}' is not a valid generation id", gen1))?;
    let b: u64 = gen2.parse()
        .with_context(|| format!("'{}' is not a valid generation id", gen2))?;

    let current = limine::current_generation();

    let pkgs_a = read_packages(a, current)
        .with_context(|| format!("could not read packages for generation {}", a))?;
    let pkgs_b = read_packages(b, current)
        .with_context(|| format!("could not read packages for generation {}", b))?;

    let names_a: HashSet<&String> = pkgs_a.keys().collect();
    let names_b: HashSet<&String> = pkgs_b.keys().collect();

    let mut added: Vec<&String> = names_b.difference(&names_a).copied().collect();
    let mut removed: Vec<&String> = names_a.difference(&names_b).copied().collect();
    added.sort();
    removed.sort();

    // Packages in both, but at different versions
    let mut upgraded: Vec<(&String, &String, &String)> = names_a
        .intersection(&names_b)
        .copied()
        .filter_map(|name| {
            let va = &pkgs_a[name];
            let vb = &pkgs_b[name];
            if va != vb { Some((name, va, vb)) } else { None }
        })
        .collect();
    upgraded.sort_by(|x, y| x.0.cmp(y.0));

    if added.is_empty() && removed.is_empty() && upgraded.is_empty() {
        println!("{} generations {} and {} are identical",
            "::".dimmed(), a.to_string().bold(), b.to_string().bold());
        return Ok(());
    }

    println!("{}", format!("Generation {} → {}", a, b).bold());
    println!("{}", "─────────────────────────────────".dimmed());

    for name in &added {
        let ver = &pkgs_b[*name];
        println!("  {} {} {}", "+".green().bold(), name.bold(), ver.dimmed());
    }
    for name in &removed {
        let ver = &pkgs_a[*name];
        println!("  {} {} {}", "-".red().bold(), name.bold(), ver.dimmed());
    }
    for (name, va, vb) in &upgraded {
        println!("  {} {} {} → {}",
            "~".blue().bold(), name.bold(), va.dimmed(), vb.green());
    }

    Ok(())
}

fn read_packages(gen: u64, current: u64) -> Result<HashMap<String, String>> {
    let db_path = if gen == current {
        PathBuf::from("/").join(DB_SUFFIX)
    } else {
        let snap = format!("{}/@gen-{}", SNAPS_DIR, gen);
        if !std::path::Path::new(&snap).exists() {
            bail!("snapshot for generation {} not found at {}", gen, snap);
        }
        PathBuf::from(&snap).join(DB_SUFFIX)
    };

    if !db_path.exists() {
        return Ok(HashMap::new());
    }

    let mut map = HashMap::new();
    for entry in fs::read_dir(&db_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if let Ok(content) = fs::read_to_string(&path) {
            // Extract version without full deserialization
            if let Some(ver) = content.lines()
                .find(|l| l.starts_with("version"))
                .and_then(|l| l.splitn(2, '=').nth(1))
                .map(|v| v.trim().trim_matches('"').to_string())
            {
                map.insert(name, ver);
            }
        }
    }
    Ok(map)
}
