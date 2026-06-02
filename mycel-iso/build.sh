#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
ISO_DIR="$BUILD_DIR/iso"
ROOTFS_DIR="$BUILD_DIR/rootfs"
SFS_DIR="$BUILD_DIR/sfs"
OUTPUT_ISO="$BUILD_DIR/MycelOS-1.0-x86_64.iso"
ISO_LABEL="MYCELOS"

# ─── Colors ───────────────────────────────────────────────────────────────────
BLUE='\033[0;34m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
step()  { echo -e "${BLUE}::${NC} $1"; }
ok()    { echo -e "${GREEN}ok${NC} $1"; }
die()   { echo -e "${RED}!!${NC} $1"; exit 1; }

# ─── Dependency check ─────────────────────────────────────────────────────────
check_deps() {
    step "checking dependencies..."
    local missing=()
    for dep in xorriso mksquashfs dracut limine; do
        command -v "$dep" &>/dev/null || missing+=("$dep")
    done
    if [ ${#missing[@]} -gt 0 ]; then
        die "missing: ${missing[*]}"
    fi
    ok "all dependencies found"
}

# ─── Bootstrap rootfs ─────────────────────────────────────────────────────────
build_rootfs() {
    step "bootstrapping rootfs..."
    bash "$SCRIPT_DIR/bootstrap.sh"
    ok "rootfs ready"
}

# ─── Create squashfs from rootfs ──────────────────────────────────────────────
create_squashfs() {
    step "creating squashfs..."
    mkdir -p "$SFS_DIR"

    # Exclude /boot — kernel and initramfs are shipped separately on the ISO
    mksquashfs "$ROOTFS_DIR" "$SFS_DIR/airootfs.sfs" \
        -comp zstd \
        -Xcompression-level 15 \
        -noappend \
        -e boot \
        2>&1 | tail -3

    ok "squashfs ready ($(du -sh "$SFS_DIR/airootfs.sfs" | cut -f1))"
}

# ─── Build kernel and initrd ──────────────────────────────────────────────────
build_boot() {
    step "copying kernel and building initrd..."
    mkdir -p "$ISO_DIR/boot"

    # Kernel is in the rootfs at /boot/vmlinuz-linux-lts (from linux-lts package)
    local kernel_path
    kernel_path=$(find "$ROOTFS_DIR/boot" -name "vmlinuz*" 2>/dev/null | head -1)
    [ -n "$kernel_path" ] || die "no kernel found in rootfs/boot — was linux-lts installed?"
    cp "$kernel_path" "$ISO_DIR/boot/vmlinuz"

    # Kernel version string, needed by dracut for module lookup
    local kver
    kver=$(ls "$ROOTFS_DIR/usr/lib/modules/" 2>/dev/null | sort -V | tail -1)
    [ -n "$kver" ] || die "no kernel modules found in rootfs/usr/lib/modules"

    # Build initramfs on the host using the rootfs's modules
    dracut --force \
        --no-hostonly \
        --add "dmsquash-live" \
        --omit "multipath iscsi fcoe nfs" \
        --kver "$kver" \
        --kmoddir "$ROOTFS_DIR/usr/lib/modules/$kver" \
        "$ISO_DIR/boot/initramfs.img"

    ok "kernel ($kver) and initrd ready"
}

# ─── Set up ISO directory structure ───────────────────────────────────────────
setup_iso_dir() {
    step "setting up ISO directory structure..."

    mkdir -p "$ISO_DIR"/{boot,EFI/BOOT,MycelOS}

    # Squashfs (built by create_squashfs above)
    cp "$SFS_DIR/airootfs.sfs" "$ISO_DIR/MycelOS/"

    # Limine bootloader files
    cp "$SCRIPT_DIR/limine/limine.conf" "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-bios.sys    "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-bios-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-uefi-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/BOOTX64.EFI        "$ISO_DIR/EFI/BOOT/"

    ok "ISO directory ready"
}

# ─── Build ISO with xorriso ───────────────────────────────────────────────────
build_iso() {
    step "building ISO with xorriso..."
    mkdir -p "$BUILD_DIR"

    xorriso -as mkisofs \
        -iso-level 3 \
        -volid "$ISO_LABEL" \
        -full-iso9660-filenames \
        \
        -b boot/limine-bios-cd.bin \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        \
        --efi-boot boot/limine-uefi-cd.bin \
        -efi-boot-part \
        --efi-boot-image \
        \
        -isohybrid-mbr /usr/share/limine/limine-bios.sys \
        \
        -output "$OUTPUT_ISO" \
        "$ISO_DIR"

    # Embed Limine into the ISO for BIOS boot
    limine bios-install "$OUTPUT_ISO"

    ok "ISO built: $OUTPUT_ISO ($(du -sh "$OUTPUT_ISO" | cut -f1))"
}

# ─── Main ─────────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo "  MycelOS ISO Builder"
    echo "  ───────────────────"
    echo ""

    check_deps
    build_rootfs
    create_squashfs
    build_boot
    setup_iso_dir
    build_iso

    echo ""
    echo -e "${GREEN}done.${NC} burn with:"
    echo "  dd if=$OUTPUT_ISO of=/dev/sdX bs=4M status=progress"
    echo ""
}

main "$@"
