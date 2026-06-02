use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::process::Command;

use crate::config::parser;
use crate::system::limine;

pub fn run() -> Result<()> {
    println!("{}", "MycelOS Doctor".bold());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();

    let mut warnings = 0usize;
    let mut errors   = 0usize;

    // ── mycel.toml ────────────────────────────────────────────────────────────
    match parser::load() {
        Ok(config) => {
            ok("mycel.toml",   "parsed ok");

            // fessus.toml for each declared user
            for user in &config.users {
                let fessus = format!("/home/{}/.config/fessus.toml", user.name);
                if std::path::Path::new(&fessus).exists() {
                    ok("fessus.toml", &format!("found for {}", user.name));
                } else {
                    warn("fessus.toml",
                         &format!("missing for {} — run: mycel edit fessus", user.name));
                    warnings += 1;
                }
            }

            // overlay sources configured
            if config.overlays.as_ref().map(|o| o.sources.is_empty()).unwrap_or(true) {
                warn("overlays", "no overlay sources — packages won't resolve");
                warnings += 1;
            } else {
                ok("overlays", &format!("{} source(s) configured",
                    config.overlays.as_ref().map(|o| o.sources.len()).unwrap_or(0)));
            }
        }
        Err(e) => {
            fail("mycel.toml", &format!("parse error: {}", e));
            errors += 1;
        }
    }

    println!();

    // ── s6-rc ─────────────────────────────────────────────────────────────────
    let live = "/run/s6-rc";
    if std::path::Path::new(live).exists() {
        let output = Command::new("s6-rc")
            .args(["-l", live, "-a", "list"])
            .output();

        match output {
            Ok(o) => {
                let services: Vec<&str> = std::str::from_utf8(&o.stdout)
                    .unwrap_or("")
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();

                ok("s6-rc", &format!("{} service(s) up: {}", services.len(), services.join(", ")));

                for expected in &["udevd", "dbus", "seatd", "pipewire", "wireplumber"] {
                    if !services.contains(expected) {
                        warn("s6-rc", &format!("core service '{}' is not up", expected));
                        warnings += 1;
                    }
                }
            }
            Err(_) => {
                warn("s6-rc", "could not query live state");
                warnings += 1;
            }
        }
    } else {
        warn("s6-rc", "live state not found at /run/s6-rc — boot may have failed");
        warnings += 1;
    }

    if !std::path::Path::new("/etc/s6-rc/compiled").exists() {
        fail("s6-rc db", "compiled database missing at /etc/s6-rc/compiled — rebuild ISO");
        errors += 1;
    } else {
        ok("s6-rc db", "compiled database present");
    }

    println!();

    // ── package DB ────────────────────────────────────────────────────────────
    let db = "/var/lib/mycel/packages";
    match fs::read_dir(db) {
        Ok(entries) => {
            let count = entries.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
                .count();
            ok("pkg db", &format!("{} package(s) registered", count));
        }
        Err(_) => {
            warn("pkg db", "package database not found — mycel switch has not run yet");
            warnings += 1;
        }
    }

    println!();

    // ── generation ────────────────────────────────────────────────────────────
    let gen = limine::current_generation();
    if gen == 0 {
        warn("generation", "no generation recorded — mycel switch has not run");
        warnings += 1;
    } else {
        ok("generation", &format!("current generation: {}", gen));
    }

    // ── disk space ────────────────────────────────────────────────────────────
    let df = Command::new("df").args(["-BM", "--output=avail", "/"]).output();
    if let Ok(o) = df {
        let avail_str = String::from_utf8_lossy(&o.stdout);
        if let Some(mb_str) = avail_str.lines().nth(1) {
            let mb: u64 = mb_str.trim().trim_end_matches('M').parse().unwrap_or(0);
            if mb < 512 {
                fail("disk", &format!("only {}MB free on / — critically low", mb));
                errors += 1;
            } else if mb < 2048 {
                warn("disk", &format!("{}MB free on / — running low", mb));
                warnings += 1;
            } else {
                ok("disk", &format!("{}MB free on /", mb));
            }
        }
    }

    println!();
    println!("{}", "─────────────────────────────────────────".dimmed());

    if errors == 0 && warnings == 0 {
        println!("{} everything looks healthy", "ok".green().bold());
    } else {
        if errors > 0 {
            println!("{} {} error(s)  {} warning(s)",
                "!!".red().bold(), errors.to_string().red().bold(),
                warnings.to_string().yellow().bold());
        } else {
            println!("{} {} warning(s)",
                "->".blue(), warnings.to_string().yellow().bold());
        }
    }

    Ok(())
}

fn ok(check: &str, detail: &str) {
    println!("  {}  {:<14} {}",
        "✓".green().bold(),
        check.bold(),
        detail.dimmed());
}

fn warn(check: &str, detail: &str) {
    println!("  {}  {:<14} {}",
        "!".yellow().bold(),
        check.bold(),
        detail);
}

fn fail(check: &str, detail: &str) {
    println!("  {}  {:<14} {}",
        "✗".red().bold(),
        check.bold(),
        detail.red());
}
