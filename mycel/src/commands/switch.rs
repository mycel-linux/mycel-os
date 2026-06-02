use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::{parser, schema::MycelConfig};

const NIX_EXPR_PATH: &str = "/etc/mycel/current.nix";
const PROFILE_PATH:  &str = "/nix/var/nix/profiles/mycel-system";
const GEN_FILE:      &str = "/etc/mycel/generation";

pub fn run() -> Result<()> {
    let pb = make_spinner();

    pb.set_message("reading /etc/mycel.toml...");
    let config = parser::load().context("could not load /etc/mycel.toml")?;

    pb.set_message("compiling nix expression...");
    let nix_expr = generate_nix_expr(&config);
    fs::create_dir_all("/etc/mycel").context("could not create /etc/mycel")?;
    fs::write(NIX_EXPR_PATH, &nix_expr).context("could not write nix expression")?;

    pb.set_message("building closures (this may take a while)...");
    let status = Command::new("nix-build")
        .args([NIX_EXPR_PATH, "-o", PROFILE_PATH])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("nix-build failed — is nix installed?")?;

    if !status.success() {
        pb.finish_and_clear();
        eprintln!("{} build failed", "!!".red().bold());
        eprintln!("   run {} to see full error output", format!("nix-build {}", NIX_EXPR_PATH).bold());
        std::process::exit(1);
    }

    pb.set_message("reloading services...");
    reload_services(&config)?;

    pb.set_message("recording generation...");
    let gen = bump_generation()?;

    pb.finish_and_clear();
    println!("{} generation {} applied", "::".blue().bold(), gen.to_string().bold());

    Ok(())
}

fn make_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn generate_nix_expr(config: &MycelConfig) -> String {
    let all_packages: HashSet<&String> = config.packages.install
        .iter()
        .chain(config.packages.lock.iter())
        .collect();

    let package_lines = {
        let mut pkgs: Vec<&String> = all_packages.into_iter().collect();
        pkgs.sort();
        pkgs.iter()
            .map(|p| format!("    {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"{{ pkgs ? import <nixpkgs> {{}} }}:
pkgs.buildEnv {{
  name = "mycel-system";
  ignoreCollisions = true;
  paths = with pkgs; [
{}
  ];
}}
"#,
        package_lines
    )
}

fn reload_services(config: &MycelConfig) -> Result<()> {
    let desired: HashSet<String> = config.services.enable.iter().cloned().collect();

    let active: HashSet<String> = fs::read_dir("/var/service")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    for svc in desired.difference(&active) {
        let src = format!("/etc/sv/{}", svc);
        let dst = format!("/var/service/{}", svc);
        if std::path::Path::new(&src).exists() {
            unix_fs::symlink(&src, &dst).ok();
        }
    }

    for svc in active.difference(&desired) {
        let dst = format!("/var/service/{}", svc);
        Command::new("sv").args(["stop", svc]).status().ok();
        fs::remove_file(&dst).ok();
    }

    Ok(())
}

fn bump_generation() -> Result<u64> {
    let current: u64 = fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);
    let next = current + 1;
    fs::write(GEN_FILE, next.to_string())?;
    Ok(next)
}
