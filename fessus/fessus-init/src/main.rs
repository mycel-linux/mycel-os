mod angles;
mod parser;
mod schema;
mod templates;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "fessus-init")]
#[command(about = "FessusDE configuration generator")]
struct Cli {
    /// Apply generated configs to ~/.config
    #[arg(long)]
    apply: bool,

    /// Print generated configs to stdout without writing
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = parser::load()?;

    if cli.dry_run {
        templates::print_all(&config)?;
        return Ok(());
    }

    templates::generate_all(&config)?;

    if cli.apply {
        println!("fessus: configuration applied");
    }

    Ok(())
}
