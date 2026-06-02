use anyhow::Result;
use colored::Colorize;

const TOPICS: &[(&str, &str)] = &[
    ("start",       "What is MycelOS and how to get started"),
    ("packages",    "Installing software with .myc recipes"),
    ("generations", "Switch, boot, rollback — the generation system"),
    ("fessus",      "Configuring FessusDE via fessus.toml"),
    ("overlays",    "Adding community package sources"),
    ("spore",       "Ephemeral shells for one-off tools"),
    ("spread",      "Exporting and restoring your setup"),
    ("packaging",   "Writing your own .myc package recipe"),
];

pub fn run(topic: Option<&str>) -> Result<()> {
    match topic {
        None    => print_index(),
        Some(t) => print_topic(t),
    }
    Ok(())
}

fn print_index() {
    println!("{}", "MycelOS Guide".bold());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();
    for (name, desc) in TOPICS {
        println!(
            "  {}  {}",
            format!("mycel guide {:<12}", name).blue().bold(),
            desc
        );
    }
    println!();
    println!("{}", "tip: run 'mycel guide start' if this is your first time".dimmed());
    println!();
}

fn print_topic(topic: &str) {
    match topic {
        "start" => {
            header("Getting Started");
            println!("MycelOS is driven by a single file:");
            println!("  {}", "/etc/mycel.toml".blue().bold());
            println!();
            println!("Everything — packages, services, users, desktop — lives there.");
            println!("When you change it, run {} to apply it.", "mycel switch".bold());
            println!();
            println!("{}", "First steps:".bold());
            println!("  1. Open your config:  {}", "mycel edit".bold());
            println!("  2. Add a package to [packages] install = [...]");
            println!("  3. Apply the change:  {}", "mycel switch".bold());
            println!();
            println!("{}", "Useful commands to know:".bold());
            println!("  {}  — see what's running", "mycel active".bold());
            println!("  {}  — list past generations", "mycel network".bold());
            println!("  {}   — full command guide", "mycel guide".bold());
            println!();
        }

        "packages" => {
            header("Installing Packages");
            println!("Packages come from {} — TOML recipe files.", ".myc".bold());
            println!();
            println!("{}", "Option 1: declare in mycel.toml (recommended)".bold());
            println!("  Add the package name to [packages] install in mycel.toml,");
            println!("  then run {}. MycelOS resolves it from your overlays.", "mycel switch".bold());
            println!();
            println!("{}", "Option 2: install a .myc recipe directly".bold());
            println!("  {}", "mycel-pkg install btop.myc".bold());
            println!();
            println!("{}", "Option 3: ephemeral — try without installing".bold());
            println!("  {}", "mycel spore btop".bold());
            println!("  The shell and everything in it vanish when you exit.");
            println!();
            println!("{}", "Pinning a package across rollbacks:".bold());
            println!("  {}", "mycel lock firefox".bold());
            println!("  Locked packages survive even if you boot into an old generation.");
            println!();
        }

        "generations" => {
            header("The Generation System");
            println!("Every time you run {}, a new generation is created.", "mycel switch".bold());
            println!("Your running system is always one specific generation.");
            println!();
            println!("{}", "Commands:".bold());
            println!("  {}     — list all generations", "mycel network".bold());
            println!("  {}  — see the current generation", "mycel active".bold());
            println!("  {}    — boot into a specific generation on next restart", "mycel boot <id>".bold());
            println!("  {} — compare two generations", "mycel diff <a> <b>".bold());
            println!();
            println!("{}", "Keeping generations:".bold());
            println!("  {}  — protect a generation from cleanup", "mycel isolate <id>".bold());
            println!("  {} — remove the protection", "mycel release <id>".bold());
            println!("  {}      — delete all unprotected old generations", "mycel purge".bold());
            println!();
        }

        "fessus" => {
            header("Configuring FessusDE");
            println!("FessusDE is configured from one file:");
            println!("  {}", "~/.config/fessus.toml".blue().bold());
            println!();
            println!("Open it:  {}", "mycel edit fessus".bold());
            println!("Changes apply to the desktop instantly when you save and close.");
            println!();
            println!("{}", "Key settings:".bold());
            println!("  accent_color  — hex color used throughout the desktop");
            println!("  theme         — dark or light");
            println!("  font          — font family name");
            println!("  wallpaper     — path to image file");
            println!();
            println!("{}", "Radial menu:".bold());
            println!("  [radial]");
            println!("  corner = \"bottom-left\"   # which corner activates the menu");
            println!("  pinned = [\"firefox\", \"kitty\", \"thunar\"]");
            println!();
            println!("{}", "Keybindings:".bold());
            println!("  [keybindings]");
            println!("  mod      = \"Super\"   # mod key (Super or Alt)");
            println!("  terminal = \"kitty\"   # app launched by mod+Return");
            println!();
        }

        "overlays" => {
            header("Community Package Overlays");
            println!("Overlays are GitHub repos that provide extra packages.");
            println!("Add them to mycel.toml:");
            println!();
            println!("  {}", "[overlays]".green());
            println!("  sources = [");
            println!("    \"github:mycel-linux/mycel-os\",");
            println!("    \"github:yourname/your-packages\",");
            println!("  ]");
            println!();
            println!("Run {} to pull in packages from new overlays.", "mycel switch".bold());
            println!();
            println!("Search the community index:");
            println!("  {}", "mycel-pkg search <query>".bold());
            println!();
            println!("To publish your own overlay, see:");
            println!("  {}", "mycel guide packaging".bold());
            println!();
        }

        "spore" => {
            header("Ephemeral Shells");
            println!("Need a tool without installing it permanently?");
            println!();
            println!("  {}", "mycel spore ffmpeg".bold());
            println!("  {}", "mycel spore python3 nodejs".bold());
            println!();
            println!("You get a shell with those packages available.");
            println!("When you exit, the environment is gone. Nothing left behind,");
            println!("no entries in your package list, no changes to your system.");
            println!();
        }

        "spread" => {
            header("Exporting Your Setup");
            println!("To copy your exact setup to another MycelOS machine:");
            println!();
            println!("  {}", "mycel spread --export ~/mybackup".bold());
            println!();
            println!("This copies mycel.toml and fessus.toml to ~/mybackup/.");
            println!();
            println!("On the new machine:");
            println!("  cp ~/mybackup/mycel.toml  /etc/mycel.toml");
            println!("  cp ~/mybackup/fessus.toml ~/.config/fessus.toml");
            println!("  {}", "mycel switch".bold());
            println!();
            println!("Your packages, services and desktop config are restored.");
            println!();
        }

        "packaging" => {
            header("Writing a .myc Package Recipe");
            println!("A .myc file is a TOML recipe that tells mycel-pkg how to install something.");
            println!();
            println!("{}", "Minimal example:".bold());
            println!("  [package]");
            println!("  name        = \"mytool\"");
            println!("  version     = \"1.0.0\"");
            println!("  description = \"does a thing\"");
            println!("  maintainer  = \"yourname\"");
            println!();
            println!("  [source]");
            println!("  type     = \"github-release\"");
            println!("  repo     = \"username/mytool\"");
            println!("  tag      = \"v1.0.0\"");
            println!("  asset    = \"mytool-x86_64-linux.tar.gz\"");
            println!("  checksum = \"sha256:...\"");
            println!();
            println!("  [install]");
            println!("  binaries = [{{ from = \"mytool\", to = \"mytool\" }}]");
            println!();
            println!("{}", "Get the checksum:".bold());
            println!("  curl -sL <download-url> | sha256sum");
            println!("  prefix the result with 'sha256:'");
            println!();
            println!("{}", "Validate your recipe:".bold());
            println!("  {}", "mycel-pkg verify mytool.myc".bold());
            println!();
            println!("{}", "Submit to the community index:".bold());
            println!("  {}", "mycel-pkg submit mytool.myc".bold());
            println!();
        }

        other => {
            println!("{} unknown topic '{}'", "!!".yellow().bold(), other);
            println!("Run {} to see available topics.", "mycel guide".bold());
        }
    }
}

fn header(title: &str) {
    println!("{}", title.bold());
    println!("{}", "─────────────────────────────────────────".dimmed());
    println!();
}
