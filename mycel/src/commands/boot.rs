use anyhow::Result;
use colored::Colorize;

pub fn run(generation: &str) -> Result<()> {
    println!("{} marking generation {} as next boot target...", "::".blue().bold(), generation.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
