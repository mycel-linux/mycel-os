use anyhow::Result;
use std::fs;

const LIMINE_CONF: &str = "/boot/limine.conf";
const GEN_FILE:    &str = "/etc/mycel/generation";
const PINS_DIR:    &str = "/etc/mycel/pins";

pub struct BootConfig<'a> {
    pub timeout: u32,
    pub extra_cmdline: &'a [String],
}

impl<'a> BootConfig<'a> {
    pub fn default_if_missing() -> Self {
        Self { timeout: 5, extra_cmdline: &[] }
    }
}

fn base_cmdline(root_dev: &str, subvol: &str, extra: &[String]) -> String {
    let extras = extra.iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let base = format!("root={} rootflags=subvol={} rw", root_dev, subvol);
    if extras.is_empty() { base } else { format!("{} {}", base, extras) }
}

/// Rewrites /boot/limine.conf with boot entries for every known generation.
pub fn write(current_gen: u64, root_dev: &str, keep: u64, boot: &BootConfig) -> Result<()> {
    let all_gens = all_generations(keep);
    let mut conf = String::new();

    conf.push_str(&format!("timeout: {}\n\n", boot.timeout));

    conf.push_str(&format!("/MycelOS — Generation {} (current)\n", current_gen));
    conf.push_str("    protocol: linux\n");
    conf.push_str("    kernel_path: root():/boot/vmlinuz\n");
    conf.push_str(&format!("    cmdline: {}\n",
        base_cmdline(root_dev, "@", boot.extra_cmdline)));
    conf.push_str("    module_path: root():/boot/initramfs.img\n\n");

    for gen in all_gens.iter().rev() {
        if *gen == current_gen { continue; }
        conf.push_str(&format!("/MycelOS — Generation {}\n", gen));
        conf.push_str("    protocol: linux\n");
        conf.push_str("    kernel_path: root():/boot/vmlinuz\n");
        conf.push_str(&format!("    cmdline: {}\n",
            base_cmdline(root_dev, &format!("/.snapshots/@gen-{}", gen), &[])));
        conf.push_str("    module_path: root():/boot/initramfs.img\n\n");
    }

    fs::create_dir_all("/boot")?;
    fs::write(LIMINE_CONF, conf)?;
    Ok(())
}

/// Puts a specific generation first in the boot menu (for `mycel boot <id>`).
pub fn set_default(target_gen: u64, root_dev: &str, keep: u64, boot: &BootConfig) -> Result<()> {
    let current  = current_generation();
    let all_gens = all_generations(keep);
    let mut conf = String::new();

    conf.push_str(&format!("timeout: {}\n\n", boot.timeout));

    let entry = |gen: u64, first: bool| -> String {
        let is_cur = gen == current;
        let label = if is_cur {
            format!("/MycelOS — Generation {} (current)", gen)
        } else {
            format!("/MycelOS — Generation {}", gen)
        };
        let subvol = if is_cur { "@".to_string() }
                     else       { format!("/.snapshots/@gen-{}", gen) };
        let cl = if first && is_cur {
            base_cmdline(root_dev, &subvol, boot.extra_cmdline)
        } else {
            base_cmdline(root_dev, &subvol, &[])
        };
        format!("{}\n    protocol: linux\n    kernel_path: root():/boot/vmlinuz\n    cmdline: {}\n    module_path: root():/boot/initramfs.img\n\n",
            label, cl)
    };

    conf.push_str(&entry(target_gen, true));
    for gen in all_gens.iter().rev() {
        if *gen == target_gen { continue; }
        conf.push_str(&entry(*gen, false));
    }

    fs::write(LIMINE_CONF, conf)?;
    Ok(())
}

pub fn current_generation() -> u64 {
    fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim().parse().unwrap_or(0)
}

fn is_pinned(gen: u64) -> bool {
    std::path::Path::new(&format!("{}/{}", PINS_DIR, gen)).exists()
}

pub fn all_generations(keep: u64) -> Vec<u64> {
    let current = current_generation();
    if current == 0 { return vec![]; }

    let mut gens: Vec<u64> = (1..=current)
        .filter(|&g| g == current || is_pinned(g) || super::btrfs::snapshot_exists(g))
        .collect();

    if gens.len() as u64 > keep {
        let unpinned: Vec<u64> = gens.iter().copied()
            .filter(|&g| g != current && !is_pinned(g)).collect();
        let to_drop = gens.len() as u64 - keep;
        let drop_set: std::collections::HashSet<u64> = unpinned.iter().copied()
            .take(to_drop as usize).collect();
        gens.retain(|g| !drop_set.contains(g));
    }
    gens
}
