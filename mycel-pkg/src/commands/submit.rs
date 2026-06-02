use anyhow::Result;
use colored::Colorize;
use crate::package::parser;
use crate::commands::verify;

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;

    println!("{} validating recipe before submission...", "::".blue().bold());
    verify::run(recipe_path)?;

    println!();
    println!("{} add this entry to community/index.toml:", "::".blue().bold());
    println!();
    println!("{}", "[[overlays]]".green());
    println!("name        = \"{}\"", recipe.package.name);
    println!("repo        = \"github:YOUR_USERNAME/YOUR_OVERLAY_REPO\"");
    println!("description = \"{}\"", recipe.package.description);
    println!("maintainer  = \"{}\"",
        recipe.package.maintainer.as_deref().unwrap_or("YOUR_USERNAME"));
    println!("verified    = false");
    println!();
    println!("{} then open a pull request at:", "->".blue());
    println!("  https://github.com/mycel-linux/mycel-os/compare");

    Ok(())
}
