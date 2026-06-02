use anyhow::Result;
use colored::Colorize;

pub fn run(package: &str) -> Result<()> {
    println!("{} removing lock from {}...", "::".blue().bold(), package.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
