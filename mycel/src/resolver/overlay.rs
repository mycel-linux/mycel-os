use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CACHE_DIR: &str = "/var/lib/mycel/overlay-cache";

pub fn fetch(source: &str, channel: &str) -> Result<PathBuf> {
    if let Some(repo) = source.strip_prefix("github:") {
        fetch_github(repo, channel)
    } else {
        anyhow::bail!("unsupported overlay source format: {}", source)
    }
}

/// Pull latest changes for all cached overlays.
pub fn update_all(channel: &str) -> Result<Vec<String>> {
    let mut updated = vec![];

    let Ok(entries) = fs::read_dir(CACHE_DIR) else {
        return Ok(updated);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }

        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let before = git_head(&path);

        if channel == "stable" {
            // Fetch tags and checkout the latest one
            Command::new("git")
                .args(["-C", path.to_str().unwrap(), "fetch", "--tags", "--quiet"])
                .status().ok();

            let tag = latest_git_tag(&path);
            if let Some(tag) = tag {
                Command::new("git")
                    .args(["-C", path.to_str().unwrap(), "checkout", &tag, "--quiet"])
                    .status().ok();
            }
        } else {
            // Pull latest main
            Command::new("git")
                .args(["-C", path.to_str().unwrap(), "pull", "--quiet"])
                .status().ok();
        }

        let after = git_head(&path);

        if before != after {
            updated.push(name);
        }
    }

    Ok(updated)
}

fn fetch_github(repo: &str, channel: &str) -> Result<PathBuf> {
    let cache_name = repo.replace('/', "-");
    let cache_path = PathBuf::from(CACHE_DIR).join(&cache_name);

    fs::create_dir_all(CACHE_DIR)?;

    if cache_path.exists() {
        // Refresh silently — if offline, use cache
        if channel == "stable" {
            Command::new("git")
                .args(["-C", cache_path.to_str().unwrap(), "fetch", "--tags", "--quiet"])
                .status().ok();
            if let Some(tag) = latest_git_tag(&cache_path) {
                Command::new("git")
                    .args(["-C", cache_path.to_str().unwrap(), "checkout", &tag, "--quiet"])
                    .status().ok();
            }
        } else {
            Command::new("git")
                .args(["-C", cache_path.to_str().unwrap(), "pull", "--quiet"])
                .status().ok();
        }
    } else {
        let url = format!("https://github.com/{}.git", repo);
        let status = Command::new("git")
            .args(["clone", "--depth=1", "--quiet", &url,
                   cache_path.to_str().unwrap()])
            .status()
            .with_context(|| format!("failed to clone {}", url))?;

        if !status.success() {
            anyhow::bail!("git clone failed for {}", url);
        }

        // On stable, checkout latest tag after clone
        if channel == "stable" {
            Command::new("git")
                .args(["-C", cache_path.to_str().unwrap(), "fetch", "--tags", "--quiet"])
                .status().ok();
            if let Some(tag) = latest_git_tag(&cache_path) {
                Command::new("git")
                    .args(["-C", cache_path.to_str().unwrap(), "checkout", &tag, "--quiet"])
                    .status().ok();
            }
        }
    }

    Ok(cache_path)
}

fn latest_git_tag(path: &PathBuf) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", path.to_str().unwrap(),
               "describe", "--tags", "--abbrev=0"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_head(path: &PathBuf) -> String {
    Command::new("git")
        .args(["-C", path.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}
