use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

const PINS_DIR: &str = "/etc/mycel/pins";

pub fn run(generation: &str) -> Result<()> {
    let gen: u64 = generation.parse()
        .with_context(|| format!("'{}' is not a valid generation id", generation))?;

    fs::create_dir_all(PINS_DIR)?;
    let pin = format!("{}/{}", PINS_DIR, gen);
    fs::write(&pin, "")?;

    println!("{} generation {} pinned — protected from purge",
        "::".blue().bold(), gen.to_string().bold());
    Ok(())
}
