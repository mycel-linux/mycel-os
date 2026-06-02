use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::config::parser;
use crate::resolver::overlay;

pub fn run() -> Result<()> {
    let config = parser::load().context("could not load /etc/mycel.toml")?;
    let channel = config.system.channel.as_deref().unwrap_or("stable");

    println!("{} channel: {}", "::".blue().bold(), channel.bold());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("pulling latest overlay cache...");

    let updated = overlay::update_all(channel)?;

    pb.finish_and_clear();

    if updated.is_empty() {
        println!("{} all overlays up to date", "ok".green().bold());
    } else {
        println!("{} updated overlays: {}", "ok".green().bold(), updated.join(", ").bold());
    }

    println!();
    println!("{} run {} to apply any new package versions",
        "->".blue(), "mycel switch".bold());

    Ok(())
}
