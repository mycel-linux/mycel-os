use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::parser;
use crate::system::{btrfs, limine};

const DB_PATH:     &str = "/var/lib/mycel/packages";
const SNAPS_DIR:   &str = "/.snapshots";

pub fn run(target_gen: Option<u64>) -> Result<()> {
    let current = limine::current_generation();
    if current == 0 {
        bail!("no generations recorded — run 'mycel switch' first");
    }

    let to = match target_gen {
        Some(g) => g,
        None => {
            if current == 1 {
                bail!("already at generation 1 — nothing to roll back to");
            }
            current - 1
        }
    };

    if to == current {
        println!("{} already at generation {}", "::".dimmed(), current);
        return Ok(());
    }
    if to > current {
        bail!("generation {} is in the future (current is {})", to, current);
    }

    let snap = format!("{}/@gen-{}", SNAPS_DIR, to);
    if !btrfs::snapshot_exists(to) {
        bail!("no snapshot for generation {} — it may have been purged", to);
    }

    println!("{} rolling back to generation {}  (current: {})",
        "::".blue().bold(), to.to_string().bold(), current.to_string().dimmed());

    // ── Live package rollback ─────────────────────────────────────────────────
    let old_pkgs = read_packages_from(&PathBuf::from(&snap).join(DB_PATH.trim_start_matches('/')));
    let cur_pkgs = read_packages_from(&PathBuf::from(DB_PATH));

    let to_install: Vec<(&String, &String)> = old_pkgs.iter()
        .filter(|(n, _)| !cur_pkgs.contains_key(*n))
        .collect();
    let to_remove: Vec<&String> = cur_pkgs.keys()
        .filter(|n| !old_pkgs.contains_key(*n))
        .collect();

    for (name, _ver) in &to_install {
        print!("  {} reinstalling {}... ", "·".dimmed(), name.bold());
        let recipe = find_recipe(name);
        let ok = if let Some(r) = recipe {
            Command::new("mycel-pkg")
                .args(["install", r.to_str().unwrap_or("")])
                .stdout(Stdio::null()).stderr(Stdio::null())
                .status().map(|s| s.success()).unwrap_or(false)
        } else { false };
        println!("{}", if ok { "ok".green() } else { "not found".yellow() });
    }

    for name in &to_remove {
        print!("  {} removing {}... ", "·".dimmed(), name.bold());
        let ok = Command::new("mycel-pkg")
            .args(["remove", name])
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().map(|s| s.success()).unwrap_or(false);
        println!("{}", if ok { "ok".green() } else { "failed".yellow() });
    }

    // ── Set boot target so next reboot uses the rolled-back generation ─────────
    if btrfs::is_btrfs_root() {
        if let Ok(root_dev) = btrfs::root_device() {
            let config = parser::load().ok();
            let keep = config.as_ref().and_then(|c| c.system.keep_generations).unwrap_or(5);
            let boot_cfg = if let Some(c) = &config {
                limine::BootConfig { timeout: c.boot.timeout, extra_cmdline: &c.boot.cmdline }
            } else {
                limine::BootConfig::default_if_missing()
            };
            limine::set_default(to, &root_dev, keep, &boot_cfg).ok();
        }
    }

    println!("{} rolled back to generation {}", "ok".green().bold(), to.to_string().bold());
    if !to_install.is_empty() || !to_remove.is_empty() {
        println!("  {} reboot to complete the rollback", "->".blue());
    }

    Ok(())
}

fn read_packages_from(db: &PathBuf) -> HashMap<String, String> {
    let Ok(entries) = fs::read_dir(db) else { return HashMap::new() };
    entries.flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension()?.to_str()? != "toml" { return None; }
            let name = path.file_stem()?.to_str()?.to_string();
            let content = fs::read_to_string(&path).ok()?;
            let ver = content.lines()
                .find(|l| l.starts_with("version"))?
                .splitn(2, '=').nth(1)?
                .trim().trim_matches('"').to_string();
            Some((name, ver))
        })
        .collect()
}

fn find_recipe(name: &str) -> Option<PathBuf> {
    let cache = PathBuf::from("/var/lib/mycel/overlay-cache");
    find_in_dir(&cache, name)
}

fn find_in_dir(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(p) = find_in_dir(&path, name) { return Some(p); }
        } else if path.extension().and_then(|e| e.to_str()) == Some("myc") {
            if path.file_stem().and_then(|s| s.to_str()) == Some(name) {
                return Some(path);
            }
        }
    }
    None
}
