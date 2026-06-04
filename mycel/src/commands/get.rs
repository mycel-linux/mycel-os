use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::parser;
use crate::resolver::PackageIndex;
use crate::system::{btrfs, limine};

const MYCEL_TOML: &str = "/etc/mycel.toml";

pub fn run(packages: &[String]) -> Result<()> {
    let config  = parser::load().context("could not load /etc/mycel.toml")?;
    let channel = config.system.channel.as_deref().unwrap_or("stable");
    let sources = config.overlays.as_ref()
        .map(|o| o.sources.clone())
        .unwrap_or_default();

    let pb = make_spinner();
    pb.set_message("resolving packages...");
    let index = PackageIndex::build(&sources, channel)?;

    let mut installed = vec![];
    let mut failed    = vec![];

    for pkg in packages {
        pb.set_message(format!("installing {}...", pkg));

        match index.find(pkg) {
            Some(recipe_path) => {
                let status = Command::new("mycel-pkg")
                    .args(["install", recipe_path.to_str().unwrap()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                match status {
                    Ok(s) if s.success() => installed.push(pkg.clone()),
                    _ => failed.push(pkg.clone()),
                }
            }
            None => {
                pb.suspend(|| {
                    eprintln!("  {} no recipe found for '{}' in any overlay",
                        "!!".yellow(), pkg);
                });
                failed.push(pkg.clone());
            }
        }
    }

    if !installed.is_empty() {
        pb.set_message("saving to mycel.toml...");
        add_to_config(&installed)
            .context("packages installed but could not update /etc/mycel.toml")?;

        // Snapshot + bump generation so the new state is recorded
        pb.set_message("recording generation...");
        let keep = config.system.keep_generations.unwrap_or(5);
        let next_gen = limine::current_generation() + 1;

        if btrfs::is_btrfs_root() {
            btrfs::snapshot(next_gen).ok();
        }

        let gen = bump_generation()?;

        if btrfs::is_btrfs_root() {
            if let Ok(root_dev) = btrfs::root_device() {
                let boot_cfg = limine::BootConfig {
                    timeout:       config.boot.timeout,
                    extra_cmdline: &config.boot.cmdline,
                };
                limine::write(gen, &root_dev, keep, &boot_cfg).ok();
            }
        }
    }

    pb.finish_and_clear();

    if !installed.is_empty() {
        println!("{} installed: {}", "ok".green().bold(),
            installed.join(", ").bold());
    }
    if !failed.is_empty() {
        println!("{} failed:    {}", "!!".yellow().bold(),
            failed.join(", ").bold());
    }

    Ok(())
}

/// Append newly-installed packages to the `packages.install` list in
/// /etc/mycel.toml, preserving the rest of the file as-is.
fn add_to_config(new_pkgs: &[String]) -> Result<()> {
    let raw = std::fs::read_to_string(MYCEL_TOML)
        .context("could not read /etc/mycel.toml")?;

    let mut doc: toml_edit::DocumentMut = raw.parse()
        .context("could not parse /etc/mycel.toml")?;

    let install = doc["packages"]["install"]
        .as_array_mut()
        .context("[packages] install is not an array")?;

    let existing: std::collections::HashSet<String> = install.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    for pkg in new_pkgs {
        if !existing.contains(pkg) {
            install.push(pkg.as_str());
        }
    }

    std::fs::write(MYCEL_TOML, doc.to_string())
        .context("could not write /etc/mycel.toml")?;

    Ok(())
}

fn bump_generation() -> Result<u64> {
    std::fs::create_dir_all("/etc/mycel")?;
    let current: u64 = std::fs::read_to_string("/etc/mycel/generation")
        .unwrap_or_else(|_| "0".to_string())
        .trim().parse().unwrap_or(0);
    let next = current + 1;
    std::fs::write("/etc/mycel/generation", next.to_string())?;
    Ok(next)
}

fn make_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
