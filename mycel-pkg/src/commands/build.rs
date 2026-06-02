use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::process::Command;

use crate::build::{checksum, compile, download, extract};
use crate::package::parser;

const TMP_DIR: &str = "/tmp/mycel-pkg-build";

pub fn run(recipe_path: &str) -> Result<()> {
    let recipe = parser::load(recipe_path)?;
    let name   = &recipe.package.name;
    let ver    = &recipe.package.version;
    let arch   = recipe.package.arch.as_deref().unwrap_or("x86_64");

    println!("{} building {} {}",
        "::".blue().bold(), name.bold(), ver.dimmed());

    let tmp = format!("{}/{}-{}", TMP_DIR, name, ver);
    fs::create_dir_all(&tmp)?;

    // ── Download source ───────────────────────────────────────────────────────
    print!("  {} downloading... ", "·".dimmed());
    let archive = download::fetch(&recipe.source, &tmp)?;
    println!("{}", "ok".green());

    // ── Verify checksum ───────────────────────────────────────────────────────
    if let Some(cs) = &recipe.source.checksum {
        print!("  {} verifying checksum... ", "·".dimmed());
        checksum::verify(&archive, cs)?;
        println!("{}", "ok".green());
    }

    // ── Extract ───────────────────────────────────────────────────────────────
    let src_dir = format!("{}/src", tmp);
    extract::extract(&archive, &src_dir)?;

    // ── Compile ───────────────────────────────────────────────────────────────
    if let Some(build) = &recipe.build {
        println!("  {} compiling...", "·".dimmed());
        compile::run(build, &src_dir)?;
    }

    // ── Stage install layout into a temporary prefix ──────────────────────────
    // We install into a staging dir and then tar it up into the .mpkg archive.
    let stage = format!("{}/stage", tmp);
    fs::create_dir_all(&stage)?;

    let prefix = recipe.install.prefix.as_deref().unwrap_or("/usr");
    let staged_prefix = format!("{}{}", stage, prefix);

    if let Some(binaries) = &recipe.install.binaries {
        fs::create_dir_all(format!("{}/bin", staged_prefix))?;
        for bin in binaries {
            let (from, to) = parse_binary_entry(bin, &src_dir);
            let dest = format!("{}/bin/{}", staged_prefix, to);
            fs::copy(&from, &dest)
                .with_context(|| format!("could not copy binary '{}' to stage", from))?;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
        }
    }

    if let Some(icons) = &recipe.install.icons {
        for icon in icons {
            let src = format!("{}/{}", src_dir, icon.src);
            let dest = format!(
                "{}/usr/share/icons/hicolor/{}x{}/apps/{}",
                stage, icon.size, icon.size, icon.name
            );
            fs::create_dir_all(std::path::Path::new(&dest).parent().unwrap())?;
            if std::path::Path::new(&src).exists() {
                fs::copy(&src, &dest)?;
            }
        }
    }

    if let Some(desktop) = &recipe.desktop {
        let dest = format!("{}/usr/share/applications/{}.desktop", stage, name);
        fs::create_dir_all(format!("{}/usr/share/applications", stage))?;
        let content = generate_desktop(name, desktop);
        fs::write(&dest, content)?;
    }

    // Write a .PKGINFO into the stage root so the archive is self-describing
    let pkginfo = format!(
        "pkgname = {}\npkgver = {}\narch = {}\n",
        name, ver, arch
    );
    fs::write(format!("{}/.PKGINFO", stage), pkginfo)?;

    // ── Package stage into .mpkg (tar.zst) ────────────────────────────────────
    let out_name = format!("{}-{}-{}.mpkg", name, ver, arch);
    let out_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(&out_name);

    print!("  {} packaging... ", "·".dimmed());
    let status = Command::new("tar")
        .args([
            "--zstd", "-cf",
            out_path.to_str().unwrap(),
            "-C", &stage, ".",
        ])
        .status()
        .context("tar not found")?;

    if !status.success() {
        anyhow::bail!("tar failed while packaging {}", name);
    }
    println!("{}", "ok".green());

    // ── Cleanup ───────────────────────────────────────────────────────────────
    fs::remove_dir_all(&tmp).ok();

    println!("{} built {} — install with: mycel-pkg install {}",
        "ok".green().bold(), name.bold(), out_path.display());
    Ok(())
}

fn parse_binary_entry(val: &toml::Value, work_dir: &str) -> (String, String) {
    match val {
        toml::Value::String(s) => (format!("{}/{}", work_dir, s), s.clone()),
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
