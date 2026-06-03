#!/bin/bash
# MycelOS ISO Builder
#
# Usage: build.sh [--profile <name>]
#   Profiles: fessus (default), plasma, gnome, cinnamon, xfce, budgie, mate, minimal
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
ROOTFS_DIR="$BUILD_DIR/rootfs"
SFS_DIR="$BUILD_DIR/sfs"

# ─── Profile selection ─────────────────────────────────────────────────────────
PROFILE="fessus"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile) PROFILE="$2"; shift 2 ;;
        *) shift ;;
    esac
done

PROFILE_FILE="$SCRIPT_DIR/profiles/${PROFILE}.sh"
[ -f "$PROFILE_FILE" ] || { echo "unknown profile '$PROFILE'"; exit 1; }
# shellcheck source=/dev/null
source "$PROFILE_FILE"

# Profile-derived values
ISO_LABEL="${PROFILE_ISO_LABEL:-MYCELOS}"
ISO_DIR="$BUILD_DIR/iso-${PROFILE}"
OUTPUT_ISO="$BUILD_DIR/MycelOS-1.0-${PROFILE}-x86_64.iso"

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
    [ ${#missing[@]} -eq 0 ] || die "missing: ${missing[*]}"
    ok "all dependencies found"
}

# ─── Bootstrap rootfs ─────────────────────────────────────────────────────────
build_rootfs() {
    step "bootstrapping rootfs (profile: ${PROFILE_NAME})..."
    bash "$SCRIPT_DIR/bootstrap.sh" --profile "$PROFILE"
    ok "rootfs ready"
}

# ─── Create squashfs ──────────────────────────────────────────────────────────
create_squashfs() {
    step "creating squashfs..."
    mkdir -p "$SFS_DIR"
    mksquashfs "$ROOTFS_DIR" "$SFS_DIR/airootfs.sfs" \
        -comp zstd -Xcompression-level 15 -noappend -e boot \
        2>&1 | tail -3
    ok "squashfs ready ($(du -sh "$SFS_DIR/airootfs.sfs" | cut -f1))"
}

# ─── Build kernel and initrd ──────────────────────────────────────────────────
build_boot() {
    step "copying kernel and building initrd..."
    mkdir -p "$ISO_DIR/boot"

    local kernel_path
    kernel_path=$(find "$ROOTFS_DIR/boot" -name "vmlinuz*" 2>/dev/null | head -1)
    [ -n "$kernel_path" ] || die "no kernel in rootfs/boot — was linux-lts installed?"
    cp "$kernel_path" "$ISO_DIR/boot/vmlinuz"

    local kver
    kver=$(ls "$ROOTFS_DIR/usr/lib/modules/" 2>/dev/null | sort -V | tail -1)
    [ -n "$kver" ] || die "no kernel modules in rootfs/usr/lib/modules"

    dracut --force \
        --no-hostonly \
        --add "dmsquash-live" \
        --omit "multipath iscsi fcoe nfs" \
        --kver "$kver" \
        --kmoddir "$ROOTFS_DIR/usr/lib/modules/$kver" \
        "$ISO_DIR/boot/initramfs.img"

    ok "kernel ($kver) and initrd ready"
}

# ─── Set up ISO directory ─────────────────────────────────────────────────────
setup_iso_dir() {
    step "setting up ISO directory structure..."
    mkdir -p "$ISO_DIR"/{boot,EFI/BOOT,MycelOS}
    cp "$SFS_DIR/airootfs.sfs"              "$ISO_DIR/MycelOS/"
    cp "$SCRIPT_DIR/limine/limine.conf"     "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-bios.sys    "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-bios-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-uefi-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/BOOTX64.EFI        "$ISO_DIR/EFI/BOOT/"
    ok "ISO directory ready"
}

# ─── Build ISO ────────────────────────────────────────────────────────────────
build_iso() {
    step "building ISO with xorriso..."
    xorriso -as mkisofs \
        -iso-level 3 \
        -volid "$ISO_LABEL" \
        -full-iso9660-filenames \
        -b boot/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot boot/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image \
        -isohybrid-mbr /usr/share/limine/limine-bios.sys \
        -output "$OUTPUT_ISO" \
        "$ISO_DIR"
    limine bios-install "$OUTPUT_ISO"
    ok "ISO built: $OUTPUT_ISO ($(du -sh "$OUTPUT_ISO" | cut -f1))"
}

# ─── Main ─────────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo "  MycelOS ISO Builder — ${PROFILE_NAME}"
    echo "  ────────────────────────────────────"
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

main
