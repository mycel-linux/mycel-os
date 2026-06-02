use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use crate::package::parser;
use crate::commands::verify;

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;
    let name   = &recipe.package.name;

    println!("{} validating {} before submission...", "::".blue().bold(), name.bold());
    println!();
    verify::run(recipe_path)?;

    println!();
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!("{} submission steps for {}:", "::".blue().bold(), name.bold());
    println!();

    let filename = Path::new(recipe_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(recipe_path);

    println!("  {}  Fork the community repo:", "1.".bold());
    println!("     https://github.com/mycel-linux/community/fork");
    println!();
    println!("  {}  Add your recipe:", "2.".bold());
    println!("     cp {} recipes/{}", recipe_path, filename);
    println!();
    println!("  {}  Commit and push:", "3.".bold());
    println!("     git add recipes/{}", filename);
    println!("     git commit -m \"add {}\"", name);
    println!("     git push");
    println!();
    println!("  {}  Open a pull request:", "4.".bold());
    println!("     https://github.com/mycel-linux/community/compare");
    println!();

    if recipe.source.checksum.is_none() && recipe.source.source_type != "git" {
        println!("{} remember to add a checksum before your PR is merged:", "->".yellow());
        println!("     curl -sL <download-url> | sha256sum");
        println!("     then add: checksum = \"sha256:<hash>\"");
        println!();
    }

    Ok(())
}
