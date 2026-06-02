use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{} scanning /nix/store for unreferenced paths...", "::".blue().bold());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
