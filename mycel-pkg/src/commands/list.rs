use anyhow::Result;
use colored::Colorize;
use crate::package::db;

pub fn run() -> Result<()> {
    let packages = db::list_all()?;

    if packages.is_empty() {
        println!("{}", "no packages installed".dimmed());
        return Ok(());
    }

    println!("{}", "Installed packages".bold());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!("  {:<24} {:<14} {}", "Name".bold(), "Version".bold(), "Installed".bold());
    println!("{}", "  ─────────────────────────────────────────".dimmed());

    for pkg in &packages {
        let date = pkg.installed_at.split('T').next().unwrap_or(&pkg.installed_at);
        println!("  {:<24} {:<14} {}", pkg.name, pkg.version, date.dimmed());
    }

    println!();
    println!("  {} package(s) installed", packages.len().to_string().blue().bold());
    Ok(())
}
