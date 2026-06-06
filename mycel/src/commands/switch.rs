use anyhow::{bail, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::commands::lock::locked_packages;
use crate::config::parser;
use crate::resolver::PackageIndex;
use crate::system::{btrfs, limine};

const GEN_FILE:  &str = "/etc/mycel/generation";
const DB_PATH:   &str = "/var/lib/mycel/packages";

pub fn run() -> Result<()> {
    let pb = make_spinner();

    // ── 1. Load config ────────────────────────────────────────────────────────
    pb.set_message("reading /etc/mycel.toml...");
    let config = parser::load().context("could not load /etc/mycel.toml")?;
    let keep = config.system.keep_generations.unwrap_or(5);

    // ── 2. Update overlay cache + build package index ─────────────────────────
    pb.set_message("updating overlay cache...");
    let channel = config.system.channel.as_deref().unwrap_or("stable");
    let sources = config.overlays
        .as_ref()
        .map(|o| o.sources.clone())
        .unwrap_or_default();
    let index = PackageIndex::build(&sources, channel)?;

    // ── 3. Diff packages ──────────────────────────────────────────────────────
    pb.set_message("resolving packages...");
    let desired: HashSet<String> = config.packages.install
        .iter()
        .chain(config.packages.lock.iter())
        .cloned()
        .collect();

    let installed: HashSet<String> = installed_packages();
    // Locked = declared in mycel.toml OR pinned via `mycel lock`
    let locked: HashSet<String> = config.packages.lock.iter().cloned()
        .chain(locked_packages())
        .collect();

    let to_install: Vec<&String> = desired.difference(&installed).collect();
    let to_remove:  Vec<&String> = installed.difference(&desired)
        .filter(|p| !locked.contains(*p))
        .collect();

    // ── 4. Install new packages ───────────────────────────────────────────────
    for pkg in &to_install {
        pb.set_message(format!("installing {}...", pkg));

        match index.find(pkg) {
            Some(recipe_path) => {
                let status = Command::new("mycel-pkg")
                    .args(["install", recipe_path.to_str().unwrap()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                match status {
                    Ok(s) if s.success() => {}
                    _ => eprintln!("  {} failed to install {} — skipping", "!!".yellow(), pkg),
                }
            }
            None => {
                eprintln!("  {} no recipe found for '{}' in any overlay", "!!".yellow(), pkg);
            }
        }
    }

    // ── 5. Remove old packages ────────────────────────────────────────────────
    for pkg in &to_remove {
        pb.set_message(format!("removing {}...", pkg));
        Command::new("mycel-pkg")
            .args(["remove", pkg])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok();
    }

    // ── 6. Apply system configuration ────────────────────────────────────────
    pb.set_message("applying system config...");
    apply_system_config(&config.system)?;

    // ── 7. Sync users ─────────────────────────────────────────────────────────
    pb.set_message("syncing users...");
    for user in &config.users {
        sync_user(user)?;
    }

    // ── 8. Recompose service definitions + hot-swap s6 services ───────────────
    // Re-weave the s6-rc database from the declarative service .toml files and
    // live-migrate the running tree to it. Non-fatal: on failure the live
    // services stay on the previous database, so we still apply the enable list.
    pb.set_message("recomposing services...");
    if let Err(e) = recompose_services() {
        eprintln!("  {} {}", "!!".yellow(), e);
    }
    pb.set_message("reloading services...");
    reload_services(&config.services.enable)?;

        // ── 9. Snapshot ───────────────────────────────────────────────────────────
    pb.set_message("snapshotting root filesystem...");
    let next_gen = limine::current_generation() + 1;

    if btrfs::is_btrfs_root() {
        match btrfs::snapshot(next_gen) {
            Ok(_)  => {},
            Err(e) => eprintln!("  {} snapshot failed (non-fatal): {}", "!!".yellow(), e),
        }
    }

    // ── 10. Record generation ─────────────────────────────────────────────────
    pb.set_message("recording generation...");
    let gen = bump_generation()?;

    // ── 11. Update Limine boot menu ───────────────────────────────────────────
    if btrfs::is_btrfs_root() {
        pb.set_message("updating boot menu...");
        if let Ok(root_dev) = btrfs::root_device() {
            let boot_cfg = limine::BootConfig {
                timeout:        config.boot.timeout,
                extra_cmdline:  &config.boot.cmdline,
            };
            limine::write(gen, &root_dev, keep, &boot_cfg).ok();
        }
    }

    // ── 12. Immutability — mark previous snapshot read-only ───────────────────
    if config.system.immutable && btrfs::is_btrfs_root() && gen > 1 {
        pb.set_message("sealing previous generation...");
        btrfs::set_snapshot_readonly(gen - 1).ok();
    }

    pb.finish_and_clear();

    println!("{} generation {} applied", "::".blue().bold(), gen.to_string().bold());

    if !to_install.is_empty() {
        println!("  {} packages installed:  {}", "+".green(),
            to_install.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }
    if !to_remove.is_empty() {
        println!("  {} packages removed:    {}", "-".red(),
            to_remove.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }

    // Show which users exist after the sync
    let user_names: Vec<&str> = config.users.iter().map(|u| u.name.as_str()).collect();
    if !user_names.is_empty() {
        println!("  {} users:               {}", "·".blue(),
            user_names.join(", ").dimmed());
    }

    // Show live service state
    if std::path::Path::new(S6_RC_LIVE).exists() {
        if let Ok(out) = Command::new("s6-rc")
            .args(["-l", S6_RC_LIVE, "-a", "list"])
            .output()
        {
            let svcs: Vec<&str> = std::str::from_utf8(&out.stdout)
                .unwrap_or("")
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            if !svcs.is_empty() {
                println!("  {} services up:         {}", "·".blue(),
                    svcs.join(", ").dimmed());
            }
        }
    }

    Ok(())
}

fn make_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn installed_packages() -> HashSet<String> {
    fs::read_dir(DB_PATH)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    if p.extension()?.to_str()? == "toml" {
                        p.file_stem()?.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

use crate::config::schema::{System, User};

fn sync_user(user: &User) -> Result<()> {
    let shell = resolve_shell(&user.shell);
    let exists = user_exists(&user.name);

    if !exists {
        // Create the user with home directory
        let mut cmd = Command::new("useradd");
        cmd.args(["-m", "-s", &shell]);
        if !user.groups.is_empty() {
            cmd.args(["-G", &user.groups.join(",")]);
        }
        cmd.arg(&user.name);
        cmd.stdout(Stdio::null()).stderr(Stdio::null()).status().ok();

        // Set password hash if provided
        if !user.password_hash.is_empty() {
            Command::new("usermod")
                .args(["-p", &user.password_hash, &user.name])
                .stdout(Stdio::null()).stderr(Stdio::null())
                .status().ok();
        }

        // Copy fessus.toml from /etc/skel or live user if present
        let user_config = format!("/home/{}/.config", user.name);
        fs::create_dir_all(&user_config).ok();
        let dest_fessus = format!("{}/fessus.toml", user_config);
        if !std::path::Path::new(&dest_fessus).exists() {
            // Try live user first, then /etc/skel
            for src in &["/home/live/.config/fessus.toml", "/etc/skel/.config/fessus.toml"] {
                if std::path::Path::new(src).exists() {
                    fs::copy(src, &dest_fessus).ok();
                    break;
                }
            }
        }
        // Fix ownership
        Command::new("chown")
            .args(["-R", &format!("{}:{}", user.name, user.name),
                   &format!("/home/{}", user.name)])
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().ok();
    } else {
        // User exists — update shell and group membership
        Command::new("usermod")
            .args(["-s", &shell, &user.name])
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().ok();

        // Add to any groups they're not already in
        for group in &user.groups {
            Command::new("usermod")
                .args(["-aG", group, &user.name])
                .stdout(Stdio::null()).stderr(Stdio::null())
                .status().ok();
        }

        // Update password hash only if non-empty (avoid locking accounts)
        if !user.password_hash.is_empty() {
            Command::new("usermod")
                .args(["-p", &user.password_hash, &user.name])
                .stdout(Stdio::null()).stderr(Stdio::null())
                .status().ok();
        }
    }

    Ok(())
}

fn user_exists(name: &str) -> bool {
    fs::read_to_string("/etc/passwd")
        .unwrap_or_default()
        .lines()
        .any(|l| l.split(':').next() == Some(name))
}

fn resolve_shell(shell: &str) -> String {
    // Accept bare names like "bash" or full paths like "/bin/bash"
    if shell.starts_with('/') {
        return shell.to_string();
    }
    for prefix in &["/bin", "/usr/bin", "/usr/local/bin"] {
        let path = format!("{}/{}", prefix, shell);
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }
    format!("/bin/{}", shell)
}

fn apply_system_config(sys: &System) -> Result<()> {
    // Hostname
    fs::write("/etc/hostname", format!("{}\n", sys.hostname))?;
    Command::new("hostname").arg(&sys.hostname).status().ok();

    // Kernel profile — set CPU frequency governor
    apply_kernel_profile(&sys.kernel);

    // Timezone — symlink /etc/localtime to the right zone file
    let zone_path = format!("/usr/share/zoneinfo/{}", sys.timezone);
    if std::path::Path::new(&zone_path).exists() {
        let _ = fs::remove_file("/etc/localtime");
        std::os::unix::fs::symlink(&zone_path, "/etc/localtime").ok();
    }

    // Locale
    fs::write("/etc/locale.conf", format!("LANG={}\n", sys.locale))?;

    // Write /etc/locale.gen and regenerate if locale-gen is available
    let locale_gen_entry = format!("{} UTF-8\n", sys.locale);
    let gen_path = "/etc/locale.gen";
    let existing = fs::read_to_string(gen_path).unwrap_or_default();
    if !existing.contains(&locale_gen_entry) {
        fs::write(gen_path, format!("{}{}", existing, locale_gen_entry))?;
    }
    if std::path::Path::new("/usr/bin/locale-gen").exists()
        || std::path::Path::new("/usr/sbin/locale-gen").exists()
    {
        Command::new("locale-gen").stdout(Stdio::null()).stderr(Stdio::null()).status().ok();
    }

    Ok(())
}

fn apply_kernel_profile(profile: &str) {
    let governor = match profile {
        "performance" => "performance",
        "battery"     => "powersave",
        "balanced"    => "schedutil",
        _             => return, // "auto" or unknown — leave as-is
    };

    // Write to every CPU's governor sysfs entry
    if let Ok(cpus) = fs::read_dir("/sys/devices/system/cpu") {
        for cpu in cpus.flatten() {
            let gov_path = cpu.path().join("cpufreq/scaling_governor");
            if gov_path.exists() {
                fs::write(&gov_path, governor).ok();
            }
        }
    }
}

const S6_RC_LIVE:    &str = "/run/s6-rc";
const S6_RC_DB:      &str = "/etc/s6-rc/compiled";
const S6_RC_SOURCE:  &str = "/etc/s6-rc/source";
const SERVICES_DIR:  &str = "/etc/mycel/services";

// Core services that must always remain up — never brought down by switch.
const ALWAYS_UP: &[&str] = &["udevd", "dbus", "seatd", "pipewire", "wireplumber"];

/// Re-weave the s6-rc database from the declarative service definitions in
/// /etc/mycel/services and live-migrate the running supervision tree to it.
///
/// This is what lets a user edit a service `.toml` and have the change take
/// effect *gracefully*: mycel-compose regenerates the s6-rc source, s6-rc-compile
/// builds a fresh database, and s6-rc-update diffs it against the live state —
/// restarting only the services whose definition actually changed, starting new
/// ones, stopping removed ones, all in dependency order, and leaving everything
/// else running untouched. No reboot, no stop-the-world.
fn recompose_services() -> Result<()> {
    use std::path::Path;

    // No declarations on this system (e.g. the live ISO) → nothing to recompose.
    if !Path::new(SERVICES_DIR).exists() {
        return Ok(());
    }
    // No composer installed → leave the build-time database as-is.
    let composer = ["/usr/bin/mycel-compose", "/usr/local/bin/mycel-compose"]
        .into_iter()
        .find(|p| Path::new(p).exists());
    let composer = match composer {
        Some(p) => p,
        None => return Ok(()),
    };

    // 1. Regenerate the s6-rc source tree from the declarations.
    let ok = Command::new(composer)
        .args(["--services", SERVICES_DIR, "--out", S6_RC_SOURCE])
        .status()
        .context("running mycel-compose")?
        .success();
    if !ok {
        bail!("mycel-compose failed — service definitions left unchanged");
    }

    // 2. Compile a fresh database beside the live one.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let new_db = format!("/etc/s6-rc/compiled-{stamp}");
    let _ = fs::remove_dir_all(&new_db); // guard against a same-second retry
    let ok = Command::new("s6-rc-compile")
        .args([new_db.as_str(), S6_RC_SOURCE])
        .status()
        .context("running s6-rc-compile")?
        .success();
    if !ok {
        bail!("s6-rc-compile failed — service definitions left unchanged");
    }

    // 3. If the supervision tree is live, migrate it gracefully. If it isn't
    //    (e.g. switch run from the installer chroot), skip — the new database is
    //    picked up at next boot via the symlink repointed below.
    if Path::new(S6_RC_LIVE).exists() {
        let ok = Command::new("s6-rc-update")
            .args(["-l", S6_RC_LIVE, new_db.as_str()])
            .status()
            .context("running s6-rc-update")?
            .success();
        if !ok {
            // Live tree is still on the old database; don't repoint anything.
            let _ = fs::remove_dir_all(&new_db);
            bail!("s6-rc-update failed — live services left on the previous database");
        }
    }

    // 4. Make /etc/s6-rc/compiled point at the new database so it survives a
    //    reboot, keeping previous databases for rollback.
    repoint_compiled_db(&new_db)?;
    Ok(())
}

/// Point the `/etc/s6-rc/compiled` symlink at `new_db`, then prune stale
/// compiled databases. The build-time layout has a real directory there; the
/// first switch retires it to `compiled-initial` (a known-good fallback) and
/// converts the path to a symlink, which every later switch swaps atomically.
fn repoint_compiled_db(new_db: &str) -> Result<()> {
    use std::path::Path;
    let link = Path::new(S6_RC_DB);

    // Retire a real build-time directory once, so the path can become a symlink.
    if link.exists() && !link.is_symlink() && link.is_dir() {
        let retired = format!("{S6_RC_DB}-initial");
        let _ = fs::remove_dir_all(&retired);
        fs::rename(link, &retired).context("retiring build-time compiled db")?;
    }

    // Atomic symlink swap: create a temp link, rename it over the target.
    let tmp = format!("{S6_RC_DB}.next");
    let _ = fs::remove_file(&tmp);
    std::os::unix::fs::symlink(new_db, &tmp).context("staging compiled-db symlink")?;
    fs::rename(&tmp, link).context("activating compiled-db symlink")?;

    prune_compiled_dbs(new_db);
    Ok(())
}

/// Keep the active timestamped database plus the two most recent previous ones
/// (for rollback); remove older `compiled-<digits>` databases. `compiled-initial`
/// is left alone as the ultimate fallback.
fn prune_compiled_dbs(keep: &str) {
    let mut dbs: Vec<(u64, std::path::PathBuf)> = match fs::read_dir("/etc/s6-rc") {
        Ok(rd) => rd
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_prefix("compiled-")
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|ts| (ts, e.path()))
            })
            .collect(),
        Err(_) => return,
    };
    dbs.sort_by(|a, b| b.0.cmp(&a.0)); // newest first

    let keep_path = std::path::Path::new(keep);
    for (_, path) in dbs.into_iter().skip(3) {
        if path != keep_path {
            let _ = fs::remove_dir_all(&path);
        }
    }
}

fn reload_services(desired: &[String]) -> Result<()> {
    // s6-rc live state doesn't exist until rc.init has run at boot.
    if !std::path::Path::new(S6_RC_LIVE).exists() {
        return Ok(());
    }

    let desired_set: HashSet<&str> = desired.iter().map(|s| s.as_str()).collect();

    let active: HashSet<String> = Command::new("s6-rc")
        .args(["-l", S6_RC_LIVE, "-a", "list"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default();

    for svc in &desired_set {
        if !active.contains(*svc) {
            Command::new("s6-rc")
                .args(["-l", S6_RC_LIVE, "-d", S6_RC_DB, "-u", "change", svc])
                .status().ok();
        }
    }

    for svc in &active {
        if !desired_set.contains(svc.as_str()) && !ALWAYS_UP.contains(&svc.as_str()) {
            Command::new("s6-rc")
                .args(["-l", S6_RC_LIVE, "-d", "change", svc])
                .status().ok();
        }
    }

    Ok(())
}

fn bump_generation() -> Result<u64> {
    fs::create_dir_all("/etc/mycel")?;
    let current: u64 = fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0);
    let next = current + 1;
    fs::write(GEN_FILE, next.to_string())?;
    Ok(next)
}
