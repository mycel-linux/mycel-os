import libcalamares
import os
import subprocess
import shutil

LIMINE_CONF = """\
timeout: 3

/MycelOS
    protocol: linux
    kernel_path: boot():/vmlinuz
    cmdline: root={root} rw quiet splash
    module_path: boot():/initramfs.img
"""


def run():
    gs      = libcalamares.globalStorage
    root    = gs.value("rootMountPoint") or "/mnt"
    root_dev = gs.value("rootDevice") or ""

    conf = LIMINE_CONF.format(root=root_dev)

    boot_dir = os.path.join(root, "boot")
    os.makedirs(boot_dir, exist_ok=True)

    conf_path = os.path.join(boot_dir, "limine.conf")
    with open(conf_path, "w") as f:
        f.write(conf)

    # Install Limine to the EFI partition
    efi_dir = os.path.join(root, "boot", "efi", "EFI", "limine")
    os.makedirs(efi_dir, exist_ok=True)

    for f in ["BOOTX64.EFI", "limine-uefi-cd.bin"]:
        src = f"/usr/share/limine/{f}"
        if os.path.exists(src):
            shutil.copy(src, efi_dir)

    disk = root_dev.rstrip("0123456789")
    subprocess.run(["limine", "bios-install", disk], check=False)

    return None
