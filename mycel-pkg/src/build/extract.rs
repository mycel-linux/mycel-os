use anyhow::{bail, Result};
use std::fs::File;
use std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;

pub fn extract(archive_path: &str, dest: &str) -> Result<()> {
    std::fs::create_dir_all(dest)?;

    let path = Path::new(archive_path);
    let name = path.file_name().unwrap_or_default().to_string_lossy();

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest)
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz") {
        extract_tar_bz2(archive_path, dest)
    } else if name.ends_with(".tar.zst") {
        extract_tar_zst(archive_path, dest)
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        extract_tar_xz(archive_path, dest)
    } else if name.ends_with(".zip") {
        extract_zip(archive_path, dest)
    } else {
        bail!("unrecognised archive format: {}", name)
    }
}

fn extract_tar_gz(path: &str, dest: &str) -> Result<()> {
    let file = File::open(path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    archive.unpack(dest)?;
    Ok(())
}

fn extract_tar_bz2(path: &str, dest: &str) -> Result<()> {
    use std::process::Command;
    let status = Command::new("tar")
        .args(["-xjf", path, "-C", dest])
        .status()?;
    if !status.success() {
        anyhow::bail!("tar failed to extract {}", path);
    }
    Ok(())
}

fn extract_tar_zst(path: &str, dest: &str) -> Result<()> {
    let file = File::open(path)?;
    let zst = zstd::Decoder::new(file)?;
    let mut archive = Archive::new(zst);
    archive.unpack(dest)?;
    Ok(())
}

fn extract_tar_xz(path: &str, dest: &str) -> Result<()> {
    use std::process::Command;
    let status = Command::new("tar")
        .args(["-xJf", path, "-C", dest])
        .status()?;
    if !status.success() {
        anyhow::bail!("tar failed to extract {}", path);
    }
    Ok(())
}

fn extract_zip(path: &str, dest: &str) -> Result<()> {
    use std::io;
    use std::os::unix::fs::PermissionsExt;

    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path = Path::new(dest).join(entry.mangled_name());

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
            if let Some(mode) = entry.unix_mode() {
                std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}
