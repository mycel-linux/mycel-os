use anyhow::{bail, Result};
use colored::Colorize;
use std::fs;
use std::process::Command;
use crate::package::{db, parser};

pub fn run(name: &str) -> Result<()> {
    let record = db::get(name)?
        .ok_or_else(|| anyhow::anyhow!("{} is not installed", name))?;

    println!("{} removing {}...", "::".blue().bold(), name.bold());

    // Remove every file the package installed
    for file in &record.files.installed {
        if fs::remove_file(file).is_ok() {
            println!("  {} {}", "✗".red(), file.dimmed());
        }
    }

    // Post-remove hook — load recipe to get hooks if it still exists
    // (best effort, don't fail if recipe is gone)
    if let Ok(recipe) = parser::load(&format!("/var/lib/mycel/recipes/{}.myc", name)) {
        if let Some(hooks) = recipe.hooks {
            if let Some(cmd) = hooks.post_remove {
                Command::new("sh").args(["-c", &cmd]).status().ok();
            }
        }
    }

    db::remove(name)?;
    println!("{} removed {}", "ok".green().bold(), name.bold());
    Ok(())
}
