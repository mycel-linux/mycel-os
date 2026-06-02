use anyhow::Result;
use colored::Colorize;

pub fn run(generation: &str) -> Result<()> {
    println!("{} pinning generation {}...", "::".blue().bold(), generation.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
