use anyhow::Result;
use colored::Colorize;

pub fn run(export_path: &str) -> Result<()> {
    println!("{} exporting config to {}...", "::".blue().bold(), export_path.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
