use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::process::Command;

const FESSUS_TOML_REL: &str = ".config/fessus.toml";

struct Theme {
    name:    &'static str,
    label:   &'static str,
    accent:  &'static str,
    mode:    &'static str,   // "dark" or "light"
}

const THEMES: &[Theme] = &[
    Theme { name: "catppuccin",    label: "Catppuccin Mocha",    accent: "#cba6f7", mode: "dark"  },
    Theme { name: "dracula",       label: "Dracula",             accent: "#bd93f9", mode: "dark"  },
    Theme { name: "nord",          label: "Nord",                accent: "#88c0d0", mode: "dark"  },
    Theme { name: "gruvbox",       label: "Gruvbox",             accent: "#b8bb26", mode: "dark"  },
    Theme { name: "tokyo-night",   label: "Tokyo Night",         accent: "#7aa2f7", mode: "dark"  },
    Theme { name: "rose-pine",     label: "Rosé Pine",           accent: "#c4a7e7", mode: "dark"  },
    Theme { name: "everforest",    label: "Everforest",          accent: "#a7c080", mode: "dark"  },
    Theme { name: "solarized",     label: "Solarized Dark",      accent: "#268bd2", mode: "dark"  },
    Theme { name: "mycel",         label: "MycelOS (default)",   accent: "#3F549E", mode: "dark"  },
    Theme { name: "latte",         label: "Catppuccin Latte",    accent: "#8839ef", mode: "light" },
    Theme { name: "light",         label: "Light",               accent: "#3F549E", mode: "light" },
];

pub fn run(name: Option<&str>) -> Result<()> {
    match name {
        None       => { list(); Ok(()) }
        Some(name) => apply(name),
    }
}

fn list() {
    println!("{}", "Available themes".bold());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();
    for t in THEMES {
        let swatch = format!("  ██  ").truecolor(
            u8::from_str_radix(&t.accent[1..3], 16).unwrap_or(127),
            u8::from_str_radix(&t.accent[3..5], 16).unwrap_or(127),
            u8::from_str_radix(&t.accent[5..7], 16).unwrap_or(127),
        );
        println!("  {}{}  {} {}",
            swatch,
            format!("{:<18}", t.name).bold(),
            t.label,
            format!("({})", t.mode).dimmed());
    }
    println!();
    println!("Apply with: {}", "mycel theme <name>".bold());
}

fn apply(name: &str) -> Result<()> {
    let theme = THEMES.iter().find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!(
            "unknown theme '{}' — run 'mycel theme' to list available themes", name
        ))?;

    let home = std::env::var("HOME").context("$HOME not set")?;
    let fessus_path = format!("{}/{}", home, FESSUS_TOML_REL);

    if !std::path::Path::new(&fessus_path).exists() {
        bail!("fessus.toml not found at {} — run 'mycel edit fessus' to create it", fessus_path);
    }

    // Use toml_edit to update accent_color and theme without touching anything else
    let raw = std::fs::read_to_string(&fessus_path)
        .context("could not read fessus.toml")?;

    let mut doc: toml_edit::DocumentMut = raw.parse()
        .context("could not parse fessus.toml")?;

    doc["fessus"]["accent_color"] = toml_edit::value(theme.accent);
    doc["fessus"]["theme"]        = toml_edit::value(theme.mode);

    std::fs::write(&fessus_path, doc.to_string())
        .context("could not write fessus.toml")?;

    // Regenerate desktop configs
    print!("{} applying {}... ", "::".blue().bold(), theme.label.bold());
    let ok = Command::new("fessus-init")
        .arg("--apply")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        println!("{}", "done".green());
    } else {
        println!("{}", "fessus-init not found — changes saved, reload manually".yellow());
    }

    Ok(())
}
