use anyhow::Result;
use colored::Colorize;
use crate::package::parser;

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;

    println!("{} validating {}...", "::".blue().bold(), recipe_path);

    let mut ok = true;

    macro_rules! check {
        ($cond:expr, $msg:expr) => {
            if $cond {
                println!("  {}  {}", "✓".green(), $msg);
            } else {
                println!("  {}  {}", "✗".red(), $msg);
                ok = false;
            }
        };
    }

    check!(!recipe.package.name.is_empty(),        "name is set");
    check!(!recipe.package.version.is_empty(),     "version is set");
    check!(!recipe.package.description.is_empty(), "description is set");
    check!(!recipe.source.source_type.is_empty(),  "source.type is set");

    check!(
        recipe.source.checksum.is_some(),
        "checksum is present (required for security)"
    );

    let has_binaries = recipe.install.binaries
        .as_ref()
        .map(|b| !b.is_empty())
        .unwrap_or(false);
    let has_script = recipe.install.script.is_some();
    check!(has_binaries || has_script, "install.binaries or install.script defined");

    if ok {
        println!("\n{} recipe is valid", "ok".green().bold());
    } else {
        println!("\n{} recipe has errors", "!!".red().bold());
        std::process::exit(1);
    }

    Ok(())
}
