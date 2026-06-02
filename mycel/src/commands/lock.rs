use anyhow::Result;
use colored::Colorize;

pub fn run(package: &str) -> Result<()> {
    println!("{} locking {} across rollbacks...", "::".blue().bold(), package.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
