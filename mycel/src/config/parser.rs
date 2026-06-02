use anyhow::{Context, Result};
use std::fs;
use super::schema::MycelConfig;

const MYCEL_TOML: &str = "/etc/mycel.toml";

pub fn load() -> Result<MycelConfig> {
    let content = fs::read_to_string(MYCEL_TOML)
        .with_context(|| format!("could not read {}", MYCEL_TOML))?;

    toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", MYCEL_TOML))
}
