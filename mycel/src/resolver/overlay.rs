use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CACHE_DIR: &str = "/var/lib/mycel/overlay-cache";

/// Fetch and cache an overlay, return the local path.
pub fn fetch(source: &str) -> Result<PathBuf> {
    if let Some(repo) = source.strip_prefix("github:") {
        fetch_github(repo)
    } else {
        anyhow::bail!("unsupported overlay source format: {}", source)
    }
}

fn fetch_github(repo: &str) -> Result<PathBuf> {
    let cache_name = repo.replace('/', "-");
    let cache_path = PathBuf::from(CACHE_DIR).join(&cache_name);

    fs::create_dir_all(CACHE_DIR)?;

    if cache_path.exists() {
        // Already cached — pull latest
        let status = Command::new("git")
            .args(["-C", cache_path.to_str().unwrap(), "pull", "--quiet"])
            .status();

        // If pull fails (no network etc.), use cached version silently
        let _ = status;
    } else {
        // Clone fresh
        let url = format!("https://github.com/{}.git", repo);
        let status = Command::new("git")
            .args(["clone", "--depth=1", "--quiet", &url,
                   cache_path.to_str().unwrap()])
            .status()
            .with_context(|| format!("failed to clone {}", url))?;

        if !status.success() {
            anyhow::bail!("git clone failed for {}", url);
        }
    }

    Ok(cache_path)
}
