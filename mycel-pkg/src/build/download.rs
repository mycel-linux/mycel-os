use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use crate::package::schema::Source;

pub fn fetch(source: &Source, dest: &str) -> Result<String> {
    let url = resolve_url(source)?;
    let filename = url.split('/').last().unwrap_or("download").to_string();
    let out_path = format!("{}/{}", dest, filename);

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.blue} downloading {msg} [{bar:30.blue}] {bytes}/{total_bytes}"
        )
        .unwrap()
        .progress_chars("█▓░"),
    );
    pb.set_message(filename.clone());

    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(&url)
        .send()
        .with_context(|| format!("failed to download {}", url))?;

    if let Some(len) = response.content_length() {
        pb.set_length(len);
    }

    let mut file = File::create(&out_path)
        .with_context(|| format!("could not create {}", out_path))?;

    let mut buf = [0u8; 8192];
    loop {
        use std::io::Read;
        let n = response.read(&mut buf)?;
        if n == 0 { break; }
        file.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }

    pb.finish_and_clear();
    Ok(out_path)
}

fn resolve_url(source: &Source) -> Result<String> {
    match source.source_type.as_str() {
        "github-release" => {
            let repo  = source.repo.as_deref().context("github-release requires 'repo'")?;
            let tag   = source.tag.as_deref().context("github-release requires 'tag'")?;
            let asset = source.asset.as_deref().context("github-release requires 'asset'")?;
            Ok(format!(
                "https://github.com/{}/releases/download/{}/{}",
                repo, tag, asset
            ))
        }
        "url" => source.url.clone().context("url source requires 'url'"),
        "git" => source.url.clone().context("git source requires 'url'"),
        other => anyhow::bail!("unknown source type: {}", other),
    }
}
