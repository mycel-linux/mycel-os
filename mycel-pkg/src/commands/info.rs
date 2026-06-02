use anyhow::Result;
use colored::Colorize;
use crate::package::{db, parser};

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;
    let pkg    = &recipe.package;

    println!("{} {}", pkg.name.bold(), pkg.version.dimmed());
    println!("{}", "─────────────────────────────────".dimmed());
    println!("  {}  {}", "description".blue(), pkg.description);

    if let Some(license) = &pkg.license {
        println!("  {}      {}", "license".blue(), license);
    }
    if let Some(maintainer) = &pkg.maintainer {
        println!("  {}  {}", "maintainer".blue(), maintainer);
    }

    println!("  {}       {}", "source".blue(), recipe.source.source_type);

    if let Some(deps) = &recipe.dependencies {
        if let Some(runtime) = &deps.runtime {
            println!("  {}     {}", "runtime".blue(), runtime.join(", "));
        }
    }

    let installed = db::is_installed(&pkg.name);
    println!(
        "  {}    {}",
        "installed".blue(),
        if installed { "yes".green().to_string() } else { "no".dimmed().to_string() }
    );

    Ok(())
}
