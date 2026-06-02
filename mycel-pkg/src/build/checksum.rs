use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;

pub fn verify(file_path: &str, expected: &str) -> Result<()> {
    let expected_hash = expected
        .strip_prefix("sha256:")
        .context("checksum must start with 'sha256:'")?;

    let mut file = File::open(file_path)
        .with_context(|| format!("could not open {}", file_path))?;

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    let actual = hex::encode(hasher.finalize());

    if actual != expected_hash {
        bail!(
            "checksum mismatch\n  expected  {}\n  got       {}",
            expected_hash, actual
        );
    }

    Ok(())
}

pub fn compute(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}
