use anyhow::Result;
use std::fs;
use std::path::Path;
use super::schema::InstalledPackage;

const DB_PATH: &str = "/var/lib/mycel/packages";

pub fn register(pkg: &InstalledPackage) -> Result<()> {
    fs::create_dir_all(DB_PATH)?;
    let path = format!("{}/{}.toml", DB_PATH, pkg.name);
    let content = toml::to_string_pretty(pkg)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let path = format!("{}/{}.toml", DB_PATH, name);
    if Path::new(&path).exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get(name: &str) -> Result<Option<InstalledPackage>> {
    let path = format!("{}/{}.toml", DB_PATH, name);
    if !Path::new(&path).exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let pkg = toml::from_str(&content)?;
    Ok(Some(pkg))
}

pub fn list_all() -> Result<Vec<InstalledPackage>> {
    fs::create_dir_all(DB_PATH)?;

    let mut packages = vec![];
    for entry in fs::read_dir(DB_PATH)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let content = fs::read_to_string(&path)?;
            if let Ok(pkg) = toml::from_str::<InstalledPackage>(&content) {
                packages.push(pkg);
            }
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

pub fn is_installed(name: &str) -> bool {
    Path::new(&format!("{}/{}.toml", DB_PATH, name)).exists()
}
