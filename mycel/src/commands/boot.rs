use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::parser;
use crate::system::{btrfs, limine};

pub fn run(generation: &str) -> Result<()> {
    let target: u64 = generation.parse()
        .with_context(|| format!("'{}' is not a valid generation id", generation))?;

    let current = limine::current_generation();
    let keep = parser::load()
        .ok()
        .and_then(|c| c.system.keep_generations)
        .unwrap_or(5);

    // Validate the target exists (current, or has a snapshot)
    if target != current && !btrfs::snapshot_exists(target) {
        bail!("generation {} has no snapshot — cannot boot into it", target);
    }

    if !btrfs::is_btrfs_root() {
        bail!("boot rollback requires a btrfs root filesystem");
    }

    let root_dev = btrfs::root_device()?;
    let boot_cfg = limine::BootConfig::default_if_missing();
    limine::set_default(target, &root_dev, keep, &boot_cfg)?;

    if target == current {
        println!("{} generation {} (current) set as default boot target",
            "::".blue().bold(), target.to_string().bold());
    } else {
        println!("{} generation {} set as default boot target",
            "::".blue().bold(), target.to_string().bold());
        println!("  {} reboot to boot into it", "->".blue());
    }

    Ok(())
}
