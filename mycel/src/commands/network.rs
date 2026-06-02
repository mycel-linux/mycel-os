use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::process::Command;

const GEN_FILE:     &str = "/etc/mycel/generation";
const PROFILE_PATH: &str = "/nix/var/nix/profiles/mycel-system";

pub fn run() -> Result<()> {
    let current: u64 = fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);

    let nix_gens = nix_generations();

    println!("{}", "Generations".bold());
    println!("{}", "─────────────────────────────────────────────────".dimmed());
    println!(
        "  {:<6} {:<22} {:<8} {}",
        "ID".bold(), "Date".bold(), "Pinned".bold(), "Size".bold()
    );
    println!("{}", "  ─────────────────────────────────────────────".dimmed());

    if nix_gens.is_empty() {
        println!("  {}", "no generations found — run 'mycel switch' first".dimmed());
    } else {
        for (id, date, size) in &nix_gens {
            let is_current = *id == current;
            let pinned = is_pinned(*id);
            let pin_str = if pinned { "yes".yellow().to_string() } else { "no".dimmed().to_string() };
            let id_str = if is_current {
                format!("{} ←", id).blue().bold().to_string()
            } else {
                id.to_string()
            };
            println!("  {:<6} {:<22} {:<8} {}", id_str, date, pin_str, size);
        }
    }

    println!();
    Ok(())
}

fn nix_generations() -> Vec<(u64, String, String)> {
    let output = Command::new("nix-env")
        .args(["--list-generations", "-p", PROFILE_PATH])
        .output();

    let Ok(output) = output else { return vec![] };
    let text = String::from_utf8_lossy(&output.stdout);

    text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 { return None; }
            let id: u64 = parts[0].parse().ok()?;
            let date = format!("{} {}", parts[1], parts[2]);
            let size = store_size(id);
            Some((id, date, size))
        })
        .collect()
}

fn store_size(id: u64) -> String {
    let path = format!("{}-{}-link", PROFILE_PATH, id);
    let output = Command::new("du")
        .args(["-sh", &path])
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

fn is_pinned(id: u64) -> bool {
    let path = format!("/etc/mycel/pins/{}", id);
    std::path::Path::new(&path).exists()
}
