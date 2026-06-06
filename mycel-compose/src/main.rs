//! mycel-compose — MycelOS declarative service composer.
//!
//! Reads a directory of per-service `.toml` declarations and weaves them into a
//! complete s6-rc source tree. One declaration in, all the supervision glue out.
//!
//!     mycel-compose --services ./services --out ./s6-rc-source
//!     mycel-compose --services ./services --check   # validate only, no output

mod generate;
mod schema;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use schema::Service;

#[derive(Parser, Debug)]
#[command(
    name = "mycel-compose",
    about = "Weave declarative service definitions into an s6-rc source tree"
)]
struct Args {
    /// Directory of `*.toml` service declarations.
    #[arg(short, long)]
    services: PathBuf,

    /// Output directory for the generated s6-rc source tree.
    #[arg(short, long, default_value = "s6-rc-source")]
    out: PathBuf,

    /// Parse and validate the declarations, but write nothing.
    #[arg(long)]
    check: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {:#}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let services = load_services(&args.services)
        .with_context(|| format!("loading declarations from {}", args.services.display()))?;

    if services.is_empty() {
        bail!("no service declarations found in {}", args.services.display());
    }

    if args.check {
        // generate() into a throwaway temp dir to exercise full validation.
        let tmp = std::env::temp_dir().join(format!("mycel-compose-check-{}", std::process::id()));
        generate::generate(&services, &tmp)?;
        fs::remove_dir_all(&tmp).ok();
        println!(
            "{} {} service declaration(s) valid",
            "ok:".green().bold(),
            services.len()
        );
        return Ok(());
    }

    generate::generate(&services, &args.out)
        .with_context(|| format!("generating s6-rc tree into {}", args.out.display()))?;

    let (longruns, oneshots, bundles) = counts(&services);
    println!(
        "{} wove {} services ({} longrun, {} oneshot, {} bundle) into {}",
        "composed:".green().bold(),
        services.len(),
        longruns,
        oneshots,
        bundles,
        args.out.display()
    );
    Ok(())
}

fn load_services(dir: &std::path::Path) -> Result<Vec<Service>> {
    let mut services = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false))
        .collect();
    entries.sort();

    for path in entries {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let svc: Service = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        services.push(svc);
    }
    Ok(services)
}

fn counts(services: &[Service]) -> (usize, usize, usize) {
    let l = services.iter().filter(|s| s.is_longrun()).count();
    let o = services.iter().filter(|s| s.is_oneshot()).count();
    let b = services.iter().filter(|s| s.is_bundle()).count();
    (l, o, b)
}
