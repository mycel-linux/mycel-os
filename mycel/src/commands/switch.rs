use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::parser;
use crate::resolver::PackageIndex;

const GEN_FILE:  &str = "/etc/mycel/generation";
const PINS_DIR:  &str = "/etc/mycel/pins";
const DB_PATH:   &str = "/var/lib/mycel/packages";

pub fn run() -> Result<()> {
    let pb = make_spinner();

    // ── 1. Load config ────────────────────────────────────────────────────────
    pb.set_message("reading /etc/mycel.toml...");
    let config = parser::load().context("could not load /etc/mycel.toml")?;

    // ── 2. Update overlay cache + build package index ─────────────────────────
    pb.set_message("updating overlay cache...");
    let sources = config.overlays
        .as_ref()
        .map(|o| o.sources.clone())
        .unwrap_or_default();
    let index = PackageIndex::build(&sources)?;

    // ── 3. Diff packages ──────────────────────────────────────────────────────
    pb.set_message("resolving packages...");
    let desired: HashSet<String> = config.packages.install
        .iter()
        .chain(config.packages.lock.iter())
        .cloned()
        .collect();

    let installed: HashSet<String> = installed_packages();
    let locked:    HashSet<String> = config.packages.lock.iter().cloned().collect();

    let to_install: Vec<&String> = desired.difference(&installed).collect();
    let to_remove:  Vec<&String> = installed.difference(&desired)
        .filter(|p| !locked.contains(*p))
        .collect();

    // ── 4. Install new packages ───────────────────────────────────────────────
    for pkg in &to_install {
        pb.set_message(format!("installing {}...", pkg));

        match index.find(pkg) {
            Some(recipe_path) => {
                let status = Command::new("mycel-pkg")
                    .args(["install", recipe_path.to_str().unwrap()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                match status {
                    Ok(s) if s.success() => {}
                    _ => eprintln!("  {} failed to install {} — skipping", "!!".yellow(), pkg),
                }
            }
            None => {
                eprintln!("  {} no recipe found for '{}' in any overlay", "!!".yellow(), pkg);
            }
        }
    }

    // ── 5. Remove old packages ────────────────────────────────────────────────
    for pkg in &to_remove {
        pb.set_message(format!("removing {}...", pkg));
        Command::new("mycel-pkg")
            .args(["remove", pkg])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok();
    }

    // ── 6. Hot-swap runit services ────────────────────────────────────────────
    pb.set_message("reloading services...");
    reload_services(&config.services.enable)?;

    // ── 7. Record generation ──────────────────────────────────────────────────
    pb.set_message("recording generation...");
    let gen = bump_generation()?;

    pb.finish_and_clear();

    println!("{} generation {} applied", "::".blue().bold(), gen.to_string().bold());
    if !to_install.is_empty() {
        println!("  {} installed: {}", "+".green(),
            to_install.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }
    if !to_remove.is_empty() {
        println!("  {} removed:   {}", "-".red(),
            to_remove.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }

    Ok(())
}

fn make_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn installed_packages() -> HashSet<String> {
    fs::read_dir(DB_PATH)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    if p.extension()?.to_str()? == "toml" {
                        p.file_stem()?.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn reload_services(desired: &[String]) -> Result<()> {
    let desired_set: HashSet<&str> = desired.iter().map(|s| s.as_str()).collect();

    let active: HashSet<String> = fs::read_dir("/var/service")
        .map(|entries| {
            entries.flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    for svc in &desired_set {
        if !active.contains(*svc) {
            let src = format!("/etc/sv/{}", svc);
            let dst = format!("/var/service/{}", svc);
            if std::path::Path::new(&src).exists() {
                unix_fs::symlink(&src, &dst).ok();
            }
        }
    }

    for svc in &active {
        if !desired_set.contains(svc.as_str()) {
            Command::new("sv").args(["stop", svc]).status().ok();
            fs::remove_file(format!("/var/service/{}", svc)).ok();
        }
    }

    Ok(())
}

fn bump_generation() -> Result<u64> {
    fs::create_dir_all("/etc/mycel")?;
    let current: u64 = fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);
    let next = current + 1;
    fs::write(GEN_FILE, next.to_string())?;
    Ok(next)
}
