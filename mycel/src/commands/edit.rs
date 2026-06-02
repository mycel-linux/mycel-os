use anyhow::{bail, Result};
use colored::Colorize;
use std::process::Command;

const MYCEL_TOML: &str = "/etc/mycel.toml";
const FESSUS_TOML: &str = "/home/.config/fessus.toml";

pub fn run(target: Option<&str>) -> Result<()> {
    let path = match target {
        Some("fessus") => FESSUS_TOML,
        Some("mycel") | None => MYCEL_TOML,
        Some(other) => bail!("unknown target '{}'. Use 'mycel' or 'fessus'.", other),
    };

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    println!("{} opening {} with {}", "::".blue().bold(), path.bold(), editor.bold());

    let status = Command::new(&editor).arg(path).status()?;

    if status.success() && target == Some("fessus") {
        println!("{} applying fessus changes...", "::".blue().bold());
        Command::new("fessus-init").arg("--apply").status()?;
        println!("{} desktop updated", "ok".green().bold());
    } else if status.success() {
        println!();
        print!("{} apply changes now? [Y/n] ", "::".blue().bold());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "n" {
            println!("{} run 'mycel switch' to apply", "->".blue());
        }
    }

    Ok(())
}
