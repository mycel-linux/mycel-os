use anyhow::Result;
use colored::Colorize;

pub fn run(packages: &[String]) -> Result<()> {
    println!("{} spawning ephemeral shell with: {}", "::".blue().bold(), packages.join(", ").bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
