use anyhow::{Context, Result};
use std::fs;
use super::schema::Recipe;

pub fn load(path: &str) -> Result<Recipe> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path))?;

    toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path))
}
