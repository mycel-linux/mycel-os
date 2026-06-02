use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::system::{btrfs, limine};

const PINS_DIR: &str = "/etc/mycel/pins";

pub fn run() -> Result<()> {
    let current = limine::current_generation();

    if current == 0 {
        println!("{}", "no generations yet — run 'mycel switch' first".dimmed());
        return Ok(());
    }

    println!("{}", "Generations".bold());
    println!("{}", "─────────────────────────────────────────────".dimmed());
    println!("  {:<6} {:<10} {:<8} {}",
        "ID".bold(), "Status".bold(), "Pinned".bold(), "Size".bold());
    println!("{}", "  ─────────────────────────────────────────────".dimmed());

    // List from newest to oldest
    for gen in (1..=current).rev() {
        let exists = gen == current || btrfs::snapshot_exists(gen);
        if !exists { continue; }

        let is_current = gen == current;
        let pinned = is_pinned(gen);

        let id_str = if is_current {
            format!("{} ←", gen).blue().bold().to_string()
        } else {
            gen.to_string()
        };

        let status = if is_current { "current".green().to_string() }
                     else          { "snapshot".dimmed().to_string() };

        let pin_str = if pinned { "yes".yellow().to_string() }
                      else      { "no".dimmed().to_string() };

        let size = if is_current { "live".dimmed().to_string() }
                   else          { btrfs::snapshot_size(gen) };

        println!("  {:<6} {:<10} {:<8} {}", id_str, status, pin_str, size);
    }

    println!();
    if !btrfs::is_btrfs_root() {
        println!("{}", "note: root is not btrfs — rollback snapshots unavailable".dimmed());
    }

    Ok(())
}

fn is_pinned(gen: u64) -> bool {
    Path::new(&format!("{}/{}", PINS_DIR, gen)).exists()
}
