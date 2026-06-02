use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;
use std::time::Duration;

pub fn run() -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("scanning /nix/store for unreferenced paths...");

    let before = store_size();

    let output = Command::new("nix-collect-garbage")
        .output()?;

    pb.finish_and_clear();

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("garbage collection failed:\n{}", err);
    }

    let after = store_size();
    let freed = parse_mb(&before).saturating_sub(parse_mb(&after));

    println!("{} store cleaned", "::".blue().bold());
    println!("  before  {}", before.dimmed());
    println!("  after   {}", after.dimmed());
    if freed > 0 {
        println!("  freed   {}", format!("~{}MB", freed).green().bold());
    }

    Ok(())
}

fn store_size() -> String {
    let output = Command::new("du")
        .args(["-sh", "/nix/store"])
        .output();

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("?")
            .to_string(),
        Err(_) => "?".to_string(),
    }
}

fn parse_mb(s: &str) -> u64 {
    let num: f64 = s.chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0);

    if s.ends_with('G') { (num * 1024.0) as u64 }
    else if s.ends_with('M') { num as u64 }
    else { 0 }
}
