use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const SPORE_ROOT: &str = "/tmp/mycel-spore";

pub fn run(packages: &[String]) -> Result<()> {
    let spore_dir = format!("{}-{}", SPORE_ROOT, std::process::id());

    println!("{} preparing ephemeral shell with: {}",
        "::".blue().bold(), packages.join(", ").bold());

    fs::create_dir_all(&spore_dir)
        .with_context(|| format!("could not create spore dir {}", spore_dir))?;

    // Install each requested package into the isolated spore root
    let mut any_installed = false;
    for pkg in packages {
        print!("  {} {} ", "·".dimmed(), pkg.bold());

        let status = Command::new("mycel-pkg")
            .env("MYCEL_ROOT", &spore_dir)
            .args(["install", pkg])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("{}", "ok".green());
                any_installed = true;
            }
            _ => println!("{}", "not found — skipping".yellow()),
        }
    }

    if !any_installed {
        fs::remove_dir_all(&spore_dir).ok();
        anyhow::bail!("no packages could be installed into the spore");
    }

    // Collect bin paths from the spore root
    let mut extra_paths = vec![
        format!("{}/usr/bin", spore_dir),
        format!("{}/usr/sbin", spore_dir),
        format!("{}/bin", spore_dir),
    ];
    extra_paths.retain(|p| std::path::Path::new(p).exists());

    // Build the new PATH: spore bins first, then the real PATH
    let current_path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
    let new_path = format!("{}:{}", extra_paths.join(":"), current_path);

    // Write a simple PS1-stamped shell rc so the user knows they're in a spore
    let rc_path = format!("{}/sporerc", spore_dir);
    fs::write(&rc_path, format!(
        "export PS1='(spore:{}) \\u@\\h:\\w\\$ '\nexport PATH='{}'\n",
        packages.join("+"), new_path,
    ))?;
    fs::set_permissions(&rc_path, fs::Permissions::from_mode(0o644))?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    println!("{} entering spore shell — type {} to exit",
        "ok".green().bold(), "exit".bold());
    println!("{}", "─────────────────────────────────".dimmed());

    let status = Command::new(&shell)
        .arg("--rcfile")
        .arg(&rc_path)
        .env("PATH", &new_path)
        .status()
        .with_context(|| format!("could not start {}", shell))?;

    println!("{}", "─────────────────────────────────".dimmed());
    println!("{} spore exited", "::".dimmed());

    // Clean up
    fs::remove_dir_all(&spore_dir).ok();

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
