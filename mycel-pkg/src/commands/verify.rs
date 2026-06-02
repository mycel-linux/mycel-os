use anyhow::Result;
use colored::Colorize;
use crate::package::parser;

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;

    println!("{} validating {}...", "::".blue().bold(), recipe_path);

    let mut errors   = 0usize;
    let mut warnings = 0usize;

    macro_rules! require {
        ($cond:expr, $msg:expr) => {
            if $cond {
                println!("  {}  {}", "✓".green(), $msg);
            } else {
                println!("  {}  {}", "✗".red(), $msg);
                errors += 1;
            }
        };
    }

    macro_rules! suggest {
        ($cond:expr, $msg:expr) => {
            if $cond {
                println!("  {}  {}", "✓".green(), $msg);
            } else {
                println!("  {}  {}", "!".yellow(), $msg);
                warnings += 1;
            }
        };
    }

    require!(!recipe.package.name.is_empty(),        "name is set");
    require!(!recipe.package.version.is_empty(),     "version is set");
    require!(!recipe.package.description.is_empty(), "description is set");
    require!(!recipe.source.source_type.is_empty(),  "source.type is set");

    let is_git = recipe.source.source_type == "git";
    suggest!(
        is_git || recipe.source.checksum.is_some(),
        if is_git {
            "git source — integrity by tag (no checksum needed)"
        } else {
            "checksum present — install will be verified"
        }
    );
    if !is_git && recipe.source.checksum.is_none() {
        println!("     {} add checksum = \"sha256:...\" before publishing", "hint:".dimmed());
    }

    let has_binaries = recipe.install.binaries
        .as_ref().map(|b| !b.is_empty()).unwrap_or(false);
    let has_script = recipe.install.script.is_some();
    require!(has_binaries || has_script, "install.binaries or install.script defined");

    suggest!(recipe.package.maintainer.is_some(), "maintainer is set");
    suggest!(recipe.package.license.is_some(),    "license is set");

    println!();
    if errors == 0 && warnings == 0 {
        println!("{} recipe is valid", "ok".green().bold());
    } else if errors == 0 {
        println!("{} recipe is valid ({} warning(s))", "ok".green().bold(), warnings);
    } else {
        println!("{} recipe has {} error(s)", "!!".red().bold(), errors);
        std::process::exit(1);
    }

    Ok(())
}
