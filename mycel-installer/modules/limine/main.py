import libcalamares
import os
import subprocess
import shutil

# kernel and initramfs live on the root partition, not the EFI partition,
# so we use root() not boot() to address them from Limine.
LIMINE_CONF = """\
timeout: 3

/MycelOS
    protocol: linux
    kernel_path: root():/boot/vmlinuz
    cmdline: root=UUID={root_uuid} rw quiet splash
    module_path: root():/boot/initramfs.img
"""


def _root_uuid(gs):
    """Return the UUID of the partition mounted at /."""
    partitions = gs.value("partitions") or []
    for p in partitions:
        if p.get("mountPoint") == "/":
            return p.get("uuid", "")
    return ""


def run():
    gs   = libcalamares.globalStorage
    root = gs.value("rootMountPoint") or "/mnt"

    root_uuid = _root_uuid(gs)
    if not root_uuid:
        return ("Could not find root partition UUID", "Limine installer cannot continue.")

    conf = LIMINE_CONF.format(root_uuid=root_uuid)

    boot_dir = os.path.join(root, "boot")
    os.makedirs(boot_dir, exist_ok=True)

    conf_path = os.path.join(boot_dir, "limine.conf")
    with open(conf_path, "w") as f:
        f.write(conf)

    # Install Limine EFI binary to the fallback EFI path so firmware finds it
    # without a registered boot entry, then also register a named entry.
    efi_limine = os.path.join(root, "boot", "efi", "EFI", "limine")
    efi_boot   = os.path.join(root, "boot", "efi", "EFI", "BOOT")
    os.makedirs(efi_limine, exist_ok=True)
    os.makedirs(efi_boot,   exist_ok=True)

    for fname in ["BOOTX64.EFI", "limine-uefi-cd.bin"]:
        src = f"/usr/share/limine/{fname}"
        if os.path.exists(src):
            shutil.copy(src, efi_limine)

    # Copy BOOTX64.EFI to the fallback path so the system boots without nvram
    bootx64 = os.path.join(efi_limine, "BOOTX64.EFI")
    if os.path.exists(bootx64):
        shutil.copy(bootx64, os.path.join(efi_boot, "BOOTX64.EFI"))

    # Register a named EFI boot entry (best-effort — may not work in all VMs)
    partitions = gs.value("partitions") or []
    efi_part   = next((p for p in partitions if p.get("mountPoint") == "/boot/efi"), None)
    if efi_part:
        disk  = efi_part.get("device", "")[:-1]   # strip partition number
        partnum = efi_part.get("device", "")[-1]
        subprocess.run(
            ["efibootmgr", "--create", "--disk", disk, "--part", partnum,
             "--label", "MycelOS", "--loader", "\\EFI\\limine\\BOOTX64.EFI"],
            check=False,
        )

    # BIOS fallback — install Limine to the MBR/VBR of the disk
    if efi_part is None:
        root_part = next((p for p in partitions if p.get("mountPoint") == "/"), None)
        if root_part:
            disk = root_part.get("device", "")[:-1]
            subprocess.run(["limine", "bios-install", disk], check=False)

    return None
