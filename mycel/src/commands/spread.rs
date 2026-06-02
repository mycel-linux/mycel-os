use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

const MYCEL_TOML:  &str = "/etc/mycel.toml";
const FESSUS_TOML: &str = ".config/fessus.toml";

pub fn run(export_path: &str) -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let fessus_src = format!("{}/{}", home, FESSUS_TOML);
    let dest = Path::new(export_path);

    fs::create_dir_all(dest)
        .with_context(|| format!("could not create {}", export_path))?;

    // Copy mycel.toml
    let mycel_dest = dest.join("mycel.toml");
    fs::copy(MYCEL_TOML, &mycel_dest)
        .with_context(|| format!("could not read {}", MYCEL_TOML))?;
    println!("{} {}", "exported".green().bold(), mycel_dest.display());

    // Copy fessus.toml if it exists
    if Path::new(&fessus_src).exists() {
        let fessus_dest = dest.join("fessus.toml");
        fs::copy(&fessus_src, &fessus_dest)
            .with_context(|| format!("could not read {}", fessus_src))?;
        println!("{} {}", "exported".green().bold(), fessus_dest.display());
    } else {
        println!("{} fessus.toml not found, skipping", "??".dimmed());
    }

    println!();
    println!("to restore on a fresh MycelOS install:");
    println!("  cp {}/mycel.toml /etc/mycel.toml", export_path);
    println!("  cp {}/fessus.toml ~/.config/fessus.toml", export_path);
    println!("  mycel switch");

    Ok(())
}
