use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct Recipe {
    pub package:      Package,
    pub source:       Source,
    pub build:        Option<Build>,
    pub install:      Install,
    pub desktop:      Option<Desktop>,
    pub dependencies: Option<Dependencies>,
    pub hooks:        Option<Hooks>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Package {
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub license:     Option<String>,
    pub maintainer:  Option<String>,
    pub arch:        Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: String,
    pub repo:        Option<String>,
    pub tag:         Option<String>,
    pub asset:       Option<String>,
    pub url:         Option<String>,
    pub checksum:    Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Build {
    pub system:   String,
    pub commands: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Install {
    pub prefix:   Option<String>,
    pub binaries: Option<Vec<toml::Value>>,
    pub icons:    Option<Vec<Icon>>,
    pub script:   Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Icon {
    pub src:  String,
    pub name: String,
    pub size: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Desktop {
    pub name:       String,
    pub exec:       String,
    pub icon:       String,
    pub categories: Option<Vec<String>>,
    pub mime:       Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dependencies {
    pub runtime:  Option<Vec<String>>,
    pub build:    Option<Vec<String>>,
    pub optional: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hooks {
    pub post_install: Option<String>,
    pub post_remove:  Option<String>,
}

// ─── Installed package record ─────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledPackage {
    pub name:         String,
    pub version:      String,
    pub installed_at: String,
    pub files:        InstalledFiles,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledFiles {
    pub installed: Vec<String>,
}
