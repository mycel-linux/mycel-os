use anyhow::Result;
use colored::Colorize;

pub fn run(gen1: &str, gen2: &str) -> Result<()> {
    println!("{} diffing generation {} → {}", "::".blue().bold(), gen1.bold(), gen2.bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
