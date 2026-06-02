use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

use super::overlay;

#[derive(Deserialize)]
struct RecipeName {
    package: PackageName,
}

#[derive(Deserialize)]
struct PackageName {
    name: String,
}

/// Maps package name → path to its .myc recipe file.
pub struct PackageIndex {
    map: HashMap<String, PathBuf>,
}

impl PackageIndex {
    /// Build an index from a list of overlay sources.
    pub fn build(sources: &[String], channel: &str) -> Result<Self> {
        let mut map = HashMap::new();

        for source in sources {
            match overlay::fetch(source, channel) {
                Ok(path) => scan_overlay(&path, &mut map),
                Err(e)   => eprintln!("  warning: skipping overlay {}: {}", source, e),
            }
        }

        Ok(Self { map })
    }

    /// Find the recipe path for a package name.
    pub fn find(&self, name: &str) -> Option<&PathBuf> {
        self.map.get(name)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

fn scan_overlay(overlay_path: &Path, map: &mut HashMap<String, PathBuf>) {
    // Look for .myc files anywhere inside the overlay directory
    scan_dir(overlay_path, map);
}

fn scan_dir(dir: &Path, map: &mut HashMap<String, PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, map);
        } else if path.extension().and_then(|e| e.to_str()) == Some("myc") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(recipe) = toml::from_str::<RecipeName>(&content) {
                    map.insert(recipe.package.name, path);
                }
            }
        }
    }
}
