#!/bin/bash
# Bootstraps the MycelOS live rootfs using Alpine Linux as the base.
# Alpine provides the system packages. MycelOS overlays its config on top.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
ROOTFS="$BUILD_DIR/rootfs"
ALPINE_VERSION="3.19"
ALPINE_ARCH="x86_64"
ALPINE_MIRROR="https://dl-cdn.alpinelinux.org/alpine"

BLUE='\033[0;34m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
step() { echo -e "${BLUE}::${NC} $1"; }
ok()   { echo -e "${GREEN}ok${NC} $1"; }
die()  { echo -e "${RED}!!${NC} $1"; exit 1; }

check_deps() {
    step "checking dependencies..."
    for dep in curl tar unsquashfs mksquashfs xorriso; do
        command -v "$dep" &>/dev/null || die "missing: $dep"
    done
    ok "dependencies found"
}

setup_alpine_base() {
    step "bootstrapping Alpine Linux base..."
    mkdir -p "$ROOTFS"

    local apk_url="$ALPINE_MIRROR/v$ALPINE_VERSION/main/$ALPINE_ARCH"
    local apk_tools_pkg
    apk_tools_pkg=$(curl -s "$apk_url/" | grep -o 'apk-tools-static[^"]*\.apk' | head -1)

    curl -sL "$apk_url/$apk_tools_pkg" -o /tmp/apk-tools.apk
    tar -xzf /tmp/apk-tools.apk -C /tmp sbin/apk.static

    /tmp/sbin/apk.static \
        -X "$apk_url" \
        -X "$ALPINE_MIRROR/v$ALPINE_VERSION/community/$ALPINE_ARCH" \
        --no-progress \
        --allow-untrusted \
        -R "$ROOTFS" \
        init

    /tmp/sbin/apk.static \
        -X "$apk_url" \
        -X "$ALPINE_MIRROR/v$ALPINE_VERSION/community/$ALPINE_ARCH" \
        --no-progress \
        --allow-untrusted \
        -R "$ROOTFS" \
        add --no-scripts \
        alpine-base \
        runit \
        eudev \
        dbus \
        seatd \
        pipewire \
        wireplumber \
        networkmanager \
        sway \
        waybar \
        dunst \
        foot \
        wofi \
        kitty \
        firefox \
        thunar \
        mousepad \
        mpv \
        imv \
        zathura \
        zathura-pdf-mupdf \
        btop \
        xarchiver \
        blueman \
        qalculate-gtk \
        wf-recorder \
        grim \
        slurp \
        wl-clipboard \
        cliphist \
        xdg-desktop-portal-wlr \
        font-inter \
        papirus-icon-theme \
        calamares \
        git \
        curl

    ok "Alpine base ready"
}

overlay_mycel() {
    step "overlaying MycelOS configuration..."

    # Copy our airootfs overlay
    cp -aT "$SCRIPT_DIR/airootfs" "$ROOTFS/"

    # Install mycel binaries
    install -Dm755 "$SCRIPT_DIR/../mycel/target/release/mycel" \
        "$ROOTFS/usr/bin/mycel"
    install -Dm755 "$SCRIPT_DIR/../mycel-pkg/target/release/mycel-pkg" \
        "$ROOTFS/usr/bin/mycel-pkg"
    install -Dm755 "$SCRIPT_DIR/../fessus/fessus-init/target/release/fessus-init" \
        "$ROOTFS/usr/bin/fessus-init"

    # Install assets
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_white.png" \
        "$ROOTFS/usr/share/mycel/logo_white.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_black.png" \
        "$ROOTFS/usr/share/mycel/logo_black.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/wallpaper.jpg" \
        "$ROOTFS/usr/share/mycel/wallpaper.jpg"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/etc/fastfetch/config.jsonc" \
        "$ROOTFS/etc/fastfetch/config.jsonc"

    # Fix fastfetch logo path
    sed -i 's|/home/tghrl/mycelos/mycel-core/assets/||g' \
        "$ROOTFS/etc/fastfetch/config.jsonc"

    # Set up live user
    chroot "$ROOTFS" adduser -D -s /bin/bash live
    chroot "$ROOTFS" sh -c "echo 'live:live' | chpasswd"
    chroot "$ROOTFS" adduser live wheel
    chroot "$ROOTFS" adduser live audio
    chroot "$ROOTFS" adduser live video
    chroot "$ROOTFS" adduser live input
    chroot "$ROOTFS" adduser live seat

    # Copy live user config
    mkdir -p "$ROOTFS/home/live/.config"
    cp -aT "$SCRIPT_DIR/airootfs/home/live/.config" \
        "$ROOTFS/home/live/.config/"

    # Run fessus-init to pre-generate DE configs
    HOME="$ROOTFS/home/live" \
        fessus-init --apply 2>/dev/null || true

    chroot "$ROOTFS" chown -R live:live /home/live

    ok "MycelOS overlay applied"
}

create_squashfs() {
    step "creating squashfs..."
    mkdir -p "$BUILD_DIR/iso/MycelOS"

    mksquashfs "$ROOTFS" "$BUILD_DIR/iso/MycelOS/airootfs.sfs" \
        -comp zstd \
        -Xcompression-level 15 \
        -noappend \
        -e boot \
        2>&1 | tail -3

    ok "squashfs ready ($(du -sh "$BUILD_DIR/iso/MycelOS/airootfs.sfs" | cut -f1))"
}

main() {
    echo ""
    echo "  MycelOS Rootfs Bootstrap"
    echo "  ────────────────────────"
    echo ""
    check_deps
    setup_alpine_base
    overlay_mycel
    create_squashfs
    echo ""
    ok "rootfs bootstrap complete — run build.sh to create the ISO"
    echo ""
}

main "$@"
