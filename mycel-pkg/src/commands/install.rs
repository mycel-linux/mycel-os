use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use chrono::Utc;

use crate::build::{checksum, download, extract};
use crate::package::{db, parser, schema::{InstalledPackage, InstalledFiles}};

const TMP_DIR: &str = "/tmp/mycel-pkg";

pub fn run(package: &str) -> Result<()> {
    let recipe = parser::load(package)?;
    let name   = &recipe.package.name;
    let ver    = &recipe.package.version;

    if db::is_installed(name) {
        println!("{} {} is already installed", "::".blue().bold(), name.bold());
        return Ok(());
    }

    println!("{} installing {} {}", "::".blue().bold(), name.bold(), ver.dimmed());

    // Download
    let tmp = format!("{}/{}-{}", TMP_DIR, name, ver);
    fs::create_dir_all(&tmp)?;

    let archive = download::fetch(&recipe.source, &tmp)?;

    // Verify checksum
    if let Some(checksum_str) = &recipe.source.checksum {
        print!("  {} verifying checksum... ", "·".dimmed());
        checksum::verify(&archive, checksum_str)?;
        println!("{}", "ok".green());
    }

    // Extract
    let src_dir = format!("{}/src", tmp);
    extract::extract(&archive, &src_dir)?;

    // Find the actual extracted directory (archives often have a subdirectory)
    let work_dir = find_work_dir(&src_dir)?;

    // Compile if source package
    if let Some(build) = &recipe.build {
        println!("  {} compiling...", "·".dimmed());
        crate::build::compile::run(build, &work_dir)?;
    }

    // Install files
    let mut installed_files: Vec<String> = vec![];
    let prefix = recipe.install.prefix.as_deref().unwrap_or("/usr");

    // Binaries
    if let Some(binaries) = &recipe.install.binaries {
        for bin in binaries {
            let (from, to) = parse_binary_entry(bin, &work_dir);
            let dest = format!("{}/bin/{}", prefix, to);
            fs::create_dir_all(format!("{}/bin", prefix))?;
            fs::copy(&from, &dest)?;
            fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
            println!("  {} {}", "→".blue(), dest.dimmed());
            installed_files.push(dest);
        }
    }

    // Icons
    if let Some(icons) = &recipe.install.icons {
        for icon in icons {
            let src = format!("{}/{}", work_dir, icon.src);
            let dest = format!(
                "/usr/share/icons/hicolor/{}x{}/apps/{}",
                icon.size, icon.size, icon.name
            );
            fs::create_dir_all(Path::new(&dest).parent().unwrap())?;
            if Path::new(&src).exists() {
                fs::copy(&src, &dest)?;
                println!("  {} {}", "→".blue(), dest.dimmed());
                installed_files.push(dest);
            }
        }
    }

    // .desktop file
    if let Some(desktop) = &recipe.desktop {
        let dest = format!("/usr/share/applications/{}.desktop", name);
        fs::create_dir_all("/usr/share/applications")?;
        let content = generate_desktop(name, desktop);
        fs::write(&dest, content)?;
        println!("  {} {}", "→".blue(), dest.dimmed());
        installed_files.push(dest);
    }

    // Post-install hook
    if let Some(hooks) = &recipe.hooks {
        if let Some(cmd) = &hooks.post_install {
            Command::new("sh").args(["-c", cmd]).status()?;
        }
    }

    // Custom install script
    if let Some(script) = &recipe.install.script {
        let script_path = format!("{}/{}", work_dir, script);
        Command::new("bash").arg(&script_path).current_dir(&work_dir).status()?;
    }

    // Register in db
    let record = InstalledPackage {
        name:         name.clone(),
        version:      ver.clone(),
        installed_at: Utc::now().to_rfc3339(),
        files:        InstalledFiles { installed: installed_files },
    };
    db::register(&record)?;

    // Cleanup
    fs::remove_dir_all(&tmp).ok();

    println!("{} installed {}", "ok".green().bold(), name.bold());
    Ok(())
}

fn find_work_dir(src_dir: &str) -> Result<String> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            return Ok(entry.path().to_string_lossy().to_string());
        }
    }
    Ok(src_dir.to_string())
}

fn parse_binary_entry(val: &toml::Value, work_dir: &str) -> (String, String) {
    match val {
        toml::Value::String(s) => {
            let full = format!("{}/{}", work_dir, s);
            (full, s.clone())
        }
        toml::Value::Table(t) => {
            let from = t.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to   = t.get("to").and_then(|v| v.as_str()).unwrap_or(from);
            (format!("{}/{}", work_dir, from), to.to_string())
        }
        _ => (String::new(), String::new()),
    }
}

fn generate_desktop(name: &str, d: &crate::package::schema::Desktop) -> String {
    let categories = d.categories.as_deref()
        .map(|c| c.join(";") + ";")
        .unwrap_or_default();
    let mime = d.mime.as_deref()
        .map(|m| format!("MimeType={}\n", m.join(";")))
        .unwrap_or_default();

    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nIcon={}\nCategories={}\n{}",
        d.name, d.exec, d.icon, categories, mime
    )
}
