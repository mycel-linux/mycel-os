use anyhow::Result;
use std::fs;

const LIMINE_CONF: &str = "/boot/limine.conf";
const GEN_FILE:    &str = "/etc/mycel/generation";
const PINS_DIR:    &str = "/etc/mycel/pins";

/// Rewrites /boot/limine.conf with a boot entry for every known generation.
/// The current generation is listed first and marked as default.
pub fn write(current_gen: u64, root_dev: &str, keep: u64) -> Result<()> {
    let all_gens = all_generations(keep);
    let mut conf = String::new();

    conf.push_str("timeout: 5\n\n");

    // Current generation — always first, boots @ subvolume (live root)
    conf.push_str(&format!("/MycelOS — Generation {} (current)\n", current_gen));
    conf.push_str("    protocol: linux\n");
    conf.push_str("    kernel_path: boot():/vmlinuz\n");
    conf.push_str(&format!(
        "    cmdline: root={} rootflags=subvol=@ rw quiet splash\n",
        root_dev
    ));
    conf.push_str("    module_path: boot():/initramfs.img\n\n");

    // Previous generations — newest first, boot their snapshot subvolume
    for gen in all_gens.iter().rev() {
        if *gen == current_gen { continue; }
        conf.push_str(&format!("/MycelOS — Generation {}\n", gen));
        conf.push_str("    protocol: linux\n");
        conf.push_str("    kernel_path: boot():/vmlinuz\n");
        conf.push_str(&format!(
            "    cmdline: root={} rootflags=subvol=/.snapshots/@gen-{} rw\n",
            root_dev, gen
        ));
        conf.push_str("    module_path: boot():/initramfs.img\n\n");
    }

    fs::create_dir_all("/boot")?;
    fs::write(LIMINE_CONF, conf)?;
    Ok(())
}

/// Rewrites limine.conf putting a specific generation first (for mycel boot <id>).
pub fn set_default(target_gen: u64, root_dev: &str, keep: u64) -> Result<()> {
    let current = current_generation();
    let all_gens = all_generations(keep);
    let mut conf = String::new();

    conf.push_str("timeout: 5\n\n");

    // Target generation first
    let is_current = target_gen == current;
    let label = if is_current {
        format!("/MycelOS — Generation {} (current)", target_gen)
    } else {
        format!("/MycelOS — Generation {}", target_gen)
    };

    conf.push_str(&format!("{}\n", label));
    conf.push_str("    protocol: linux\n");
    conf.push_str("    kernel_path: boot():/vmlinuz\n");
    if is_current {
        conf.push_str(&format!(
            "    cmdline: root={} rootflags=subvol=@ rw quiet splash\n",
            root_dev
        ));
    } else {
        conf.push_str(&format!(
            "    cmdline: root={} rootflags=subvol=/.snapshots/@gen-{} rw\n",
            root_dev, target_gen
        ));
    }
    conf.push_str("    module_path: boot():/initramfs.img\n\n");

    // All other generations
    for gen in all_gens.iter().rev() {
        if *gen == target_gen { continue; }
        let is_cur = *gen == current;
        let lbl = if is_cur {
            format!("/MycelOS — Generation {} (current)", gen)
        } else {
            format!("/MycelOS — Generation {}", gen)
        };
        conf.push_str(&format!("{}\n", lbl));
        conf.push_str("    protocol: linux\n");
        conf.push_str("    kernel_path: boot():/vmlinuz\n");
        if is_cur {
            conf.push_str(&format!(
                "    cmdline: root={} rootflags=subvol=@ rw quiet splash\n",
                root_dev
            ));
        } else {
            conf.push_str(&format!(
                "    cmdline: root={} rootflags=subvol=/.snapshots/@gen-{} rw\n",
                root_dev, gen
            ));
        }
        conf.push_str("    module_path: boot():/initramfs.img\n\n");
    }

    fs::write(LIMINE_CONF, conf)?;
    Ok(())
}

pub fn current_generation() -> u64 {
    fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse()
        .unwrap_or(0)
}

fn is_pinned(gen: u64) -> bool {
    std::path::Path::new(&format!("{}/{}", PINS_DIR, gen)).exists()
}

/// Returns all generations that should appear in the boot menu.
pub fn all_generations(keep: u64) -> Vec<u64> {
    let current = current_generation();
    if current == 0 { return vec![]; }

    let mut gens: Vec<u64> = (1..=current)
        .filter(|&g| {
            g == current
                || is_pinned(g)
                || super::btrfs::snapshot_exists(g)
        })
        .collect();

    // Trim to keep limit (always keep pinned and current)
    if gens.len() as u64 > keep {
        let unpinned: Vec<u64> = gens.iter()
            .copied()
            .filter(|&g| g != current && !is_pinned(g))
            .collect();
        let to_drop = gens.len() as u64 - keep;
        let drop_set: std::collections::HashSet<u64> = unpinned
            .iter()
            .copied()
            .take(to_drop as usize)
            .collect();
        gens.retain(|g| !drop_set.contains(g));
    }

    gens
}
