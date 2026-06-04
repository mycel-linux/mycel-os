use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::config::parser;
use crate::system::{btrfs, limine};

const PINS_DIR: &str = "/etc/mycel/pins";

pub fn run() -> Result<()> {
    if !btrfs::is_btrfs_root() {
        println!("{} root is not btrfs — no generation snapshots to purge",
            "::".dimmed());
        return Ok(());
    }

    let current = limine::current_generation();
    let config = parser::load().ok();
    let keep = config
        .as_ref()
        .and_then(|c| c.system.keep_generations)
        .unwrap_or(5);

    // Collect existing snapshot generations, oldest first
    let mut snapshots: Vec<u64> = (1..current)
        .filter(|&g| btrfs::snapshot_exists(g))
        .collect();
    snapshots.sort_unstable();

    // Generations we're allowed to delete: unpinned, and beyond the keep window
    let pinned: Vec<u64> = snapshots.iter()
        .copied()
        .filter(|&g| is_pinned(g))
        .collect();

    let deletable: Vec<u64> = snapshots.iter()
        .copied()
        .filter(|&g| !is_pinned(g))
        .collect();

    // Keep the newest `keep` generations (current always counts as one)
    let keep_unpinned = keep.saturating_sub(1) as usize;
    let to_delete: Vec<u64> = if deletable.len() > keep_unpinned {
        deletable[..deletable.len() - keep_unpinned].to_vec()
    } else {
        vec![]
    };

    if to_delete.is_empty() {
        println!("{} nothing to purge", "::".blue().bold());
        println!("  {} generation(s) kept, {} pinned",
            snapshots.len() + 1, pinned.len());
        return Ok(());
    }

    println!("{} purging {} old generation(s)...",
        "::".blue().bold(), to_delete.len().to_string().bold());

    for gen in &to_delete {
        match btrfs::delete_snapshot(*gen) {
            Ok(_)  => println!("  {} generation {}", "✗".red(), gen),
            Err(e) => eprintln!("  {} failed to delete gen {}: {}", "!!".yellow(), gen, e),
        }
    }

    // Rewrite the boot menu without the deleted generations
    if let Ok(root_dev) = btrfs::root_device() {
        let boot_cfg = match &config {
            Some(c) => limine::BootConfig { timeout: c.boot.timeout, extra_cmdline: &c.boot.cmdline },
            None    => limine::BootConfig::default_if_missing(),
        };
        limine::write(current, &root_dev, keep, &boot_cfg).ok();
    }

    println!("{} purge complete", "ok".green().bold());
    Ok(())
}

fn is_pinned(gen: u64) -> bool {
    Path::new(&format!("{}/{}", PINS_DIR, gen)).exists()
}
