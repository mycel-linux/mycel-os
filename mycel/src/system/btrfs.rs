use anyhow::Result;
use std::fs;
use std::process::Command;

const SNAPSHOTS_DIR: &str = "/.snapshots";

/// Returns true if the root filesystem is btrfs.
pub fn is_btrfs_root() -> bool {
    fs::read_to_string("/proc/mounts")
        .unwrap_or_default()
        .lines()
        .any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() >= 3 && parts[1] == "/" && parts[2] == "btrfs"
        })
}

/// Get the root block device (e.g. /dev/sda2).
pub fn root_device() -> Result<String> {
    let output = Command::new("findmnt")
        .args(["-n", "-o", "SOURCE", "/"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create a snapshot of the current root before applying a new generation.
pub fn snapshot(gen: u64) -> Result<()> {
    fs::create_dir_all(SNAPSHOTS_DIR)?;
    let dest = format!("{}/@gen-{}", SNAPSHOTS_DIR, gen);
    let status = Command::new("btrfs")
        .args(["subvolume", "snapshot", "-r", "/", &dest])
        .status()?;
    if !status.success() {
        anyhow::bail!("btrfs snapshot failed for generation {}", gen);
    }
    Ok(())
}

/// Delete a snapshot for a given generation.
pub fn delete_snapshot(gen: u64) -> Result<()> {
    let path = format!("{}/@gen-{}", SNAPSHOTS_DIR, gen);
    if std::path::Path::new(&path).exists() {
        let status = Command::new("btrfs")
            .args(["subvolume", "delete", &path])
            .status()?;
        if !status.success() {
            anyhow::bail!("btrfs delete failed for generation {}", gen);
        }
    }
    Ok(())
}

/// Returns the disk usage of a snapshot in human-readable form.
pub fn snapshot_size(gen: u64) -> String {
    let path = format!("{}/@gen-{}", SNAPSHOTS_DIR, gen);
    Command::new("du")
        .args(["-sh", &path])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("?")
                .to_string()
        })
        .unwrap_or_else(|_| "?".to_string())
}

/// Returns true if a snapshot exists for this generation.
pub fn snapshot_exists(gen: u64) -> bool {
    std::path::Path::new(&format!("{}/@gen-{}", SNAPSHOTS_DIR, gen)).exists()
}
