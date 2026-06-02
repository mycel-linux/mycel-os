mod commands;
mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "mycel")]
#[command(version = "0.1.0")]
#[command(about = "MycelOS system manager — the underground network")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate mycel.toml and apply it to the running system
    Switch,

    /// Set the boot generation for next restart
    Boot {
        /// Generation ID to boot into
        generation: String,
    },

    /// Open mycel.toml or fessus.toml in $EDITOR
    Edit {
        /// Target config to edit: mycel (default) or fessus
        target: Option<String>,
    },

    /// List all system generations
    Network,

    /// Alias for network
    Grid,

    /// Show the currently running system state
    Active {
        /// Print only the generation ID
        #[arg(long)]
        gen: bool,
    },

    /// Show the diff between two generations
    Diff {
        gen1: String,
        gen2: String,
    },

    /// Garbage collect the /nix/store
    Purge,

    /// Alias for purge
    Gc,

    /// Pin a generation so it survives garbage collection
    Isolate {
        generation: String,
    },

    /// Unpin a generation
    Release {
        generation: String,
    },

    /// Pin a package so it survives rollbacks
    Lock {
        package: String,
    },

    /// Remove a package pin
    Unlock {
        package: String,
    },

    /// Drop into an ephemeral shell with the given packages
    Spore {
        /// Packages to include in the shell
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// Export your config for use on a fresh install
    Spread {
        /// Path to export mycel.toml and fessus.toml into
        #[arg(long)]
        export: String,
    },

    /// Show the built-in guide for new users
    Guide {
        /// Topic to read: start, packages, generations, fessus, spore, spread
        topic: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Switch           => commands::switch::run(),
        Commands::Boot { generation }  => commands::boot::run(&generation),
        Commands::Edit { target }      => commands::edit::run(target.as_deref()),
        Commands::Network | Commands::Grid  => commands::network::run(),
        Commands::Active { gen }       => commands::active::run(gen),
        Commands::Diff { gen1, gen2 }  => commands::diff::run(&gen1, &gen2),
        Commands::Purge | Commands::Gc => commands::purge::run(),
        Commands::Isolate { generation } => commands::isolate::run(&generation),
        Commands::Release { generation } => commands::release::run(&generation),
        Commands::Lock { package }     => commands::lock::run(&package),
        Commands::Unlock { package }   => commands::unlock::run(&package),
        Commands::Spore { packages }   => commands::spore::run(&packages),
        Commands::Spread { export }    => commands::spread::run(&export),
        Commands::Guide { topic }      => commands::guide::run(topic.as_deref()),
    }
}
