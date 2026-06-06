import libcalamares
import os
import glob
import shutil
import subprocess

# kernel + initramfs live on the root partition (root() in Limine terms).
LIMINE_CONF = """\
timeout: 3

/MycelOS
    protocol: linux
    kernel_path: root():/boot/vmlinuz
    cmdline: root=UUID={root_uuid} rw quiet splash
    module_path: root():/boot/initramfs.img
"""


def _gs():
    return libcalamares.globalStorage


def _partitions():
    return _gs().value("partitions") or []


def _uuid_for(mountpoint):
    for p in _partitions():
        if p.get("mountPoint") == mountpoint:
            return p.get("uuid", "")
    return ""


def _device_for(mountpoint):
    for p in _partitions():
        if p.get("mountPoint") == mountpoint:
            return p.get("device", "")
    return ""


def _chroot(root, args):
    """Run a command inside the target system."""
    return subprocess.run(["chroot", root] + args, check=False)


def run():
    gs   = _gs()
    root = gs.value("rootMountPoint") or "/mnt"

    root_uuid = _uuid_for("/")
    if not root_uuid:
        return ("Bootloader", "Could not find the root partition UUID.")

    # ── kernel + initramfs ────────────────────────────────────────────────────
    # The squashfs excludes /boot, and the live initramfs is dmsquash-live (wrong
    # for a disk boot). So: copy the kernel out of the modules dir and generate a
    # fresh, normal initramfs for the installed system via dracut in the chroot.
    os.makedirs(os.path.join(root, "boot"), exist_ok=True)

    modules_dir = os.path.join(root, "usr/lib/modules")
    kvers = sorted(os.listdir(modules_dir)) if os.path.isdir(modules_dir) else []
    if not kvers:
        return ("Bootloader", "No kernel modules found in the target.")
    kver = kvers[-1]

    vmlinuz = os.path.join(modules_dir, kver, "vmlinuz")
    if os.path.exists(vmlinuz):
        shutil.copy(vmlinuz, os.path.join(root, "boot", "vmlinuz"))
    else:
        return ("Bootloader", "vmlinuz not found in the target modules dir.")

    # depmod + a normal (non-live) initramfs inside the target
    _chroot(root, ["depmod", kver])
    _chroot(root, [
        "dracut", "--force", "--no-hostonly",
        "--omit", "dmsquash-live",
        "--kver", kver, "/boot/initramfs.img",
    ])

    # ── limine.conf ───────────────────────────────────────────────────────────
    with open(os.path.join(root, "boot", "limine.conf"), "w") as f:
        f.write(LIMINE_CONF.format(root_uuid=root_uuid))

    # ── install limine (UEFI and/or BIOS) ─────────────────────────────────────
    # Limine data files live in the target's /usr/share/limine.
    limine_share = os.path.join(root, "usr/share/limine")
    efi_dev = _device_for("/boot/efi")

    if efi_dev:
        efi_limine = os.path.join(root, "boot/efi/EFI/limine")
        efi_boot   = os.path.join(root, "boot/efi/EFI/BOOT")
        os.makedirs(efi_limine, exist_ok=True)
        os.makedirs(efi_boot,   exist_ok=True)

        bootx64 = os.path.join(limine_share, "BOOTX64.EFI")
        if os.path.exists(bootx64):
            shutil.copy(bootx64, os.path.join(efi_limine, "BOOTX64.EFI"))
            # Fallback path so it boots even without an NVRAM entry
            shutil.copy(bootx64, os.path.join(efi_boot, "BOOTX64.EFI"))

        # Register an NVRAM entry (best-effort; harmless if it fails in a VM)
        disk   = efi_dev.rstrip("0123456789").rstrip("p")
        partnum = "".join(c for c in efi_dev if c.isdigit())[-1:] or "1"
        _chroot(root, [
            "efibootmgr", "--create", "--disk", disk, "--part", partnum,
            "--label", "MycelOS", "--loader", "\\EFI\\limine\\BOOTX64.EFI",
        ])
    else:
        # BIOS install onto the disk holding root
        root_dev = _device_for("/")
        disk = root_dev.rstrip("0123456789").rstrip("p")
        # limine bios-install needs limine-bios.sys reachable on the partition
        bios_sys = os.path.join(limine_share, "limine-bios.sys")
        if os.path.exists(bios_sys):
            shutil.copy(bios_sys, os.path.join(root, "boot", "limine-bios.sys"))
        _chroot(root, ["limine", "bios-install", disk])

    return None
