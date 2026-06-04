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
PROFILE="plasma"
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

    # Determine the kernel version from the installed modules directory.
    local kver
    kver=$(ls "$ROOTFS_DIR/usr/lib/modules/" 2>/dev/null | sort -V | tail -1)
    [ -n "$kver" ] || die "no kernel modules in rootfs/usr/lib/modules — was linux-lts installed?"

    # Modern Arch kernels ship vmlinuz inside the modules dir; the /boot copy is
    # normally made by a pacman hook that never runs in our extraction-based build.
    local kernel_path="$ROOTFS_DIR/usr/lib/modules/$kver/vmlinuz"
    [ -f "$kernel_path" ] || kernel_path=$(find "$ROOTFS_DIR/boot" -name 'vmlinuz*' 2>/dev/null | head -1)
    [ -n "$kernel_path" ] && [ -f "$kernel_path" ] || die "no kernel image found for $kver"
    cp "$kernel_path" "$ISO_DIR/boot/vmlinuz"

    # Generate modules.dep etc. — normally done by a pacman hook that doesn't
    # run in our extraction-based build, so dracut would fail without it.
    step "running depmod for $kver..."
    depmod -b "$ROOTFS_DIR" "$kver" || die "depmod failed for $kver"

    # Explicitly bundle the drivers a live ISO needs: the CD/disk controllers
    # to reach the medium, iso9660 to read it, loop+squashfs for the image, and
    # overlay/dm for the writable root. --no-hostonly alone has proven not to
    # pull all of these reliably for an offline-built generic initramfs.
    dracut --force \
        --no-hostonly \
        --add "dmsquash-live" \
        --add-drivers "squashfs loop overlay dm_snapshot dm_mod iso9660 \
                       sr_mod cdrom sd_mod ahci ata_piix \
                       virtio_blk virtio_scsi virtio_pci" \
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
    cp "$SFS_DIR/airootfs.sfs" "$ISO_DIR/MycelOS/"

    # Generate limine.conf with the profile's CDLABEL substituted in so the
    # initramfs can find the squashfs by volume label.
    sed "s/@CDLABEL@/${ISO_LABEL}/g" \
        "$SCRIPT_DIR/limine/limine.conf" > "$ISO_DIR/boot/limine.conf"

    cp /usr/share/limine/limine-bios.sys    "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-bios-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/limine-uefi-cd.bin "$ISO_DIR/boot/"
    cp /usr/share/limine/BOOTX64.EFI        "$ISO_DIR/EFI/BOOT/"
    ok "ISO directory ready"
}

# ─── Build ISO ────────────────────────────────────────────────────────────────
build_iso() {
    step "building ISO with xorriso..."
    # Modern limine (v4+) hybrid ISO: use --protective-msdos-label, NOT
    # -isohybrid-mbr. The latter conflicts with -efi-boot-part and produces
    # "Overlapping MBR partition entries". limine bios-install embeds the BIOS
    # stage into the finished ISO afterward.
    xorriso -as mkisofs \
        -R -r -J \
        -iso-level 3 \
        -volid "$ISO_LABEL" \
        -b boot/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot boot/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image \
        --protective-msdos-label \
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

    # Fast-iteration flags:
    #   MYCEL_SKIP_BOOTSTRAP=1  reuse the existing rootfs (skip package install)
    #   MYCEL_SKIP_SQUASHFS=1   reuse the existing squashfs (skip recompression)
    # Use SKIP_BOOTSTRAP alone after editing files in the rootfs — the squashfs
    # is rebuilt so the changes make it into the image.
    if [ "${MYCEL_SKIP_BOOTSTRAP:-0}" = "1" ] && [ -d "$ROOTFS_DIR" ]; then
        step "MYCEL_SKIP_BOOTSTRAP=1 — reusing existing rootfs"
    else
        build_rootfs
    fi

    if [ "${MYCEL_SKIP_SQUASHFS:-0}" = "1" ] && [ -f "$SFS_DIR/airootfs.sfs" ]; then
        step "MYCEL_SKIP_SQUASHFS=1 — reusing existing squashfs"
    else
        create_squashfs
    fi

    build_boot
    setup_iso_dir
    build_iso

    echo ""
    echo -e "${GREEN}done.${NC} burn with:"
    echo "  dd if=$OUTPUT_ISO of=/dev/sdX bs=4M status=progress"
    echo ""
}

main
