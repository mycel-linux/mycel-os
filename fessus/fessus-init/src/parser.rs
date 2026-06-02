use anyhow::{Context, Result};
use std::fs;
use super::schema::FessusConfig;

pub fn load() -> Result<FessusConfig> {
    let path = shellexpand_tilde("~/.config/fessus.toml");

    let content = fs::read_to_string(&path)
        .with_context(|| format!("could not read {}", path))?;

    toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path))
}

fn shellexpand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        format!("{}/{}", home, &path[2..])
    } else {
        path.to_string()
    }
}
