use anyhow::Result;
use colored::Colorize;
use crate::package::parser;

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;
    println!("{} building {} {}...", "::".blue().bold(),
        recipe.package.name.bold(), recipe.package.version.dimmed());
    println!("{} not yet implemented — use 'mycel-pkg install' for pre-built packages",
        "!!".yellow().bold());
    Ok(())
}
