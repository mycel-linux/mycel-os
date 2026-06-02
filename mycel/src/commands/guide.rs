use anyhow::Result;
use colored::Colorize;

const TOPICS: &[(&str, &str)] = &[
    ("start",       "What is MycelOS and how to get started"),
    ("packages",    "Adding and removing software via mycel.toml"),
    ("generations", "Switch, boot, gc — the generation system"),
    ("fessus",      "Configuring FessusDE via fessus.toml"),
    ("spore",       "Ephemeral shells for one-off tools"),
    ("spread",      "Exporting and restoring your setup"),
];

pub fn run(topic: Option<&str>) -> Result<()> {
    match topic {
        None => print_index(),
        Some(t) => print_topic(t),
    }
    Ok(())
}

fn print_index() {
    println!("{}", "MycelOS Guide".bold());
    println!("{}", "─────────────────────────────────".dimmed());
    println!();
    for (name, desc) in TOPICS {
        println!("  {}  {}", format!("mycel guide {}", name).blue().bold(), desc);
    }
    println!();
}

fn print_topic(topic: &str) {
    match topic {
        "start" => {
            println!("{}", "Getting Started".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Your entire system is declared in one file:");
            println!("  {}", "/etc/mycel.toml".blue().bold());
            println!();
            println!("Open it with:  {}", "mycel edit".bold());
            println!("Apply changes: {}", "mycel switch".bold());
            println!();
            println!("To install an app, add it to [packages] install = [...] and run mycel switch.");
            println!();
        }
        "packages" => {
            println!("{}", "Managing Packages".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Add a package:    edit [packages] install = [...] in mycel.toml, then {}", "mycel switch".bold());
            println!("Lock a package:   {}", "mycel lock <package>".bold());
            println!("Ephemeral shell:  {}", "mycel spore <package>".bold());
            println!();
        }
        "generations" => {
            println!("{}", "The Generation System".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Every time you run {} a new generation is created.", "mycel switch".bold());
            println!("List generations:  {}", "mycel network".bold());
            println!("Boot into one:     {}", "mycel boot <id>".bold());
            println!("Pin one:           {}", "mycel isolate <id>".bold());
            println!("Clean old ones:    {}", "mycel purge".bold());
            println!();
        }
        "fessus" => {
            println!("{}", "Configuring FessusDE".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Your desktop config lives at: {}", "~/.config/fessus.toml".blue().bold());
            println!("Open it with: {}", "mycel edit fessus".bold());
            println!("Changes apply instantly when you save and close.");
            println!();
        }
        "spore" => {
            println!("{}", "Ephemeral Shells".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Need a tool without installing it permanently?");
            println!("  {}", "mycel spore ffmpeg".bold());
            println!();
            println!("The shell and its packages vanish when you exit. Nothing is left behind.");
            println!();
        }
        "spread" => {
            println!("{}", "Exporting Your Setup".bold());
            println!("{}", "─────────────────────────────────".dimmed());
            println!();
            println!("Copy your config to a new MycelOS machine:");
            println!("  {}", "mycel spread --export ~/mybackup".bold());
            println!();
            println!("Then on the new machine, drop the files in place and run {}", "mycel switch".bold());
            println!();
        }
        other => {
            println!("{} unknown topic '{}'", "!!".yellow().bold(), other);
            println!("Run {} to see available topics.", "mycel guide".bold());
        }
    }
}
