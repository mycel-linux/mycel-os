use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use chrono::Utc;

use crate::build::{checksum, download, extract};
use crate::package::{db, parser, schema::{InstalledPackage, InstalledFiles}};
use crate::root::system_root;

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

    // AppImage — handle separately: place in /opt/appimages/, create wrapper
    if is_appimage(&archive) {
        let installed_files = install_appimage(&archive, name, &recipe, &tmp)?;
        let record = InstalledPackage {
            name: name.clone(), version: ver.clone(),
            installed_at: Utc::now().to_rfc3339(),
            files: InstalledFiles { installed: installed_files },
        };
        db::register(&record)?;
        fs::remove_dir_all(&tmp).ok();
        println!("{} installed {}", "ok".green().bold(), name.bold());
        return Ok(());
    }

    // Extract
    let src_dir = format!("{}/src", tmp);
    extract::extract(&archive, &src_dir)?;

    // Use src_dir as the root — recipes specify full paths from archive root
    let work_dir = src_dir.clone();

    // Compile if source package
    if let Some(build) = &recipe.build {
        println!("  {} compiling...", "·".dimmed());
        crate::build::compile::run(build, &work_dir)?;
    }

    // Install files
    let root = system_root();
    let mut installed_files: Vec<String> = vec![];
    let prefix = format!("{}{}", root, recipe.install.prefix.as_deref().unwrap_or("/usr"));

    // Binaries
    if let Some(binaries) = &recipe.install.binaries {
        for bin in binaries {
            let (from, to) = parse_binary_entry(bin, &work_dir);
            let dest = format!("{}/bin/{}", prefix, to);
            fs::create_dir_all(format!("{}/bin", prefix))?;
            fs::copy(&from, &dest)
                .with_context(|| format!("could not copy '{}' — check the 'from' path in the recipe", from))?;
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
                "{}/usr/share/icons/hicolor/{}x{}/apps/{}",
                root, icon.size, icon.size, icon.name
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
        let dest = format!("{}/usr/share/applications/{}.desktop", root, name);
        fs::create_dir_all(format!("{}/usr/share/applications", root))?;
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

fn is_appimage(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.ends_with(".appimage") { return true; }
    // Check magic bytes: ELF + AppImage type 2 marker at offset 8
    if let Ok(mut f) = std::fs::File::open(path) {
        use std::io::Read;
        let mut buf = [0u8; 12];
        if f.read_exact(&mut buf).is_ok() {
            // ELF magic + AI\x02 at offset 8
            return buf[0..4] == [0x7f, b'E', b'L', b'F']
                && buf[8..11] == [0x41, 0x49, 0x02];
        }
    }
    false
}

fn install_appimage(
    src: &str,
    name: &str,
    recipe: &crate::package::schema::Recipe,
    tmp: &str,
) -> Result<Vec<String>> {
    let root    = system_root();
    let appdir  = format!("{}/opt/appimages", root);
    let bindir  = format!("{}/usr/bin", root);
    let appfile = format!("{}/{}.AppImage", appdir, name);
    let wrapper = format!("{}/{}", bindir, name);

    fs::create_dir_all(&appdir)?;
    fs::create_dir_all(&bindir)?;
    fs::copy(src, &appfile)?;
    fs::set_permissions(&appfile, fs::Permissions::from_mode(0o755))?;

    // Wrapper script so the binary appears in PATH normally
    fs::write(&wrapper, format!(
        "#!/bin/sh\nexec {appfile} \"$@\"\n",
        appfile = appfile
    ))?;
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755))?;
    println!("  {} {}", "→".blue(), wrapper.dimmed());
    println!("  {} {}", "→".blue(), appfile.dimmed());

    let mut installed = vec![appfile.clone(), wrapper.clone()];

    // Try to extract .desktop and icon from the AppImage
    let extract_dir = format!("{}/appimage-extract", tmp);
    let extracted = Command::new(&appfile)
        .arg("--appimage-extract")
        .current_dir(tmp)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if extracted {
        let squash = format!("{}/squashfs-root", tmp);

        // Copy .desktop file
        if let Ok(entries) = fs::read_dir(&squash) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    let dest = format!("{}/usr/share/applications/{}.desktop", root, name);
                    fs::create_dir_all(format!("{}/usr/share/applications", root)).ok();
                    fs::copy(&p, &dest).ok();
                    installed.push(dest);
                    break;
                }
            }
        }

        // Copy icon (look for .png or .svg in squashfs-root)
        for icon_name in &[
            format!("{}.png", name),
            format!("{}.svg", name),
            ".DirIcon".to_string(),
        ] {
            let icon_src = format!("{}/{}", squash, icon_name);
            if Path::new(&icon_src).exists() {
                let ext = if icon_src.ends_with(".svg") { "svg" } else { "png" };
                let dest = format!("{}/usr/share/icons/hicolor/256x256/apps/{}.{}",
                    root, name, ext);
                fs::create_dir_all(std::path::Path::new(&dest).parent().unwrap()).ok();
                fs::copy(&icon_src, &dest).ok();
                installed.push(dest);
                break;
            }
        }
    }

    // Fall back to recipe desktop entry if extraction didn't produce one
    if let Some(desktop) = &recipe.desktop {
        let dest = format!("{}/usr/share/applications/{}.desktop", root, name);
        if !Path::new(&dest).exists() {
            fs::create_dir_all(format!("{}/usr/share/applications", root)).ok();
            fs::write(&dest, generate_desktop(name, desktop)).ok();
            installed.push(dest);
        }
    }

    Ok(installed)
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
