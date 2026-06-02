mod commands;
mod package;
mod build;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mycel-pkg")]
#[command(version = "0.1.0")]
#[command(about = "MycelOS package manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a .mpkg binary from a .myc recipe
    Build {
        recipe: String,
    },
    /// Install a package from a .myc recipe or .mpkg file
    Install {
        package: String,
    },
    /// Remove an installed package
    Remove {
        name: String,
    },
    /// Validate a .myc recipe without installing
    Verify {
        recipe: String,
    },
    /// Show package metadata
    Info {
        recipe: String,
    },
    /// List all installed packages
    List,
    /// Search the community index
    Search {
        query: String,
    },
    /// Validate and prepare a recipe for community submission
    Submit {
        recipe: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build   { recipe }  => commands::build::run(&recipe),
        Commands::Install { package } => commands::install::run(&package),
        Commands::Remove  { name }    => commands::remove::run(&name),
        Commands::Verify  { recipe }  => commands::verify::run(&recipe),
        Commands::Info    { recipe }  => commands::info::run(&recipe),
        Commands::List                => commands::list::run(),
        Commands::Search  { query }   => commands::search::run(&query),
        Commands::Submit  { recipe }  => commands::submit::run(&recipe),
    }
}
