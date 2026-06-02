use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "Generation  Date                 Pinned  Size".bold());
    println!("{}", "─────────────────────────────────────────────".dimmed());
    println!("{} not yet implemented", "!!".yellow().bold());
    Ok(())
}
