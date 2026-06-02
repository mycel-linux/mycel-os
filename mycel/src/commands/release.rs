use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

const PINS_DIR: &str = "/etc/mycel/pins";

pub fn run(generation: &str) -> Result<()> {
    let gen: u64 = generation.parse()
        .with_context(|| format!("'{}' is not a valid generation id", generation))?;

    let pin = format!("{}/{}", PINS_DIR, gen);
    if Path::new(&pin).exists() {
        fs::remove_file(&pin)?;
        println!("{} generation {} unpinned — may be reclaimed on next purge",
            "::".blue().bold(), gen.to_string().bold());
    } else {
        println!("{} generation {} was not pinned",
            "::".dimmed(), gen.to_string().bold());
    }
    Ok(())
}
