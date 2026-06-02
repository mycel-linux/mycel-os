#!/bin/bash
# MycelOS Live ISO Rootfs Bootstrap
#
# Builds the live filesystem from scratch.
# Arch Linux packages are used as a BUILD-TIME binary source only.
# The final rootfs runs on runit + mycel-pkg — no Arch tooling is present.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/build/rootfs"
PKG_CACHE="$SCRIPT_DIR/build/pkg-cache"
RECIPES="$SCRIPT_DIR/../community/recipes"
MYCEL_PKG="$SCRIPT_DIR/../mycel-pkg/target/release/mycel-pkg"
FESSUS_INIT="$SCRIPT_DIR/../fessus/fessus-init/target/release/fessus-init"

BUSYBOX_URL="https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox"
MUSL_URL="https://musl.libc.org/releases/musl-1.2.5.tar.gz"
ARCH_MIRROR="https://geo.mirror.pkgbuild.com/extra/os/x86_64"

BLUE='\033[0;34m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
step() { echo -e "\n${BLUE}::${NC} $1"; }
ok()   { echo -e "   ${GREEN}ok${NC} $1"; }
die()  { echo -e "   ${RED}!!${NC} $1"; exit 1; }
info() { echo -e "   → $1"; }

# ─── Preflight ────────────────────────────────────────────────────────────────

check_deps() {
    step "checking build dependencies..."
    for dep in curl tar zstd mksquashfs gcc make; do
        command -v "$dep" &>/dev/null || die "missing: $dep"
    done
    [ -x "$MYCEL_PKG" ]   || die "mycel-pkg not built — run: cd mycel-pkg && cargo build --release"
    [ -x "$FESSUS_INIT" ] || die "fessus-init not built — run: cd fessus/fessus-init && cargo build --release"
    ok "dependencies ok"
}

# ─── 1. Filesystem skeleton ───────────────────────────────────────────────────

create_skeleton() {
    step "creating filesystem skeleton..."
    rm -rf "$ROOT"
    mkdir -p "$ROOT"/{bin,sbin,lib,lib64,
                     usr/{bin,sbin,lib,lib64,share/{applications,icons,fonts,mycel},include,local},
                     etc/{sv,runit/{1,2,3},mycel,fastfetch,sway,waybar,dunst},
                     var/{lib/mycel/packages,log,run,tmp},
                     proc,sys,dev/{pts,shm},run,tmp,
                     home/live/.config,root,boot,mnt,media,opt}
    chmod 1777 "$ROOT/tmp"
    chmod 0750 "$ROOT/root"
    mkdir -p "$PKG_CACHE"
    ok "directory tree ready"
}

# ─── 2. Busybox ───────────────────────────────────────────────────────────────

install_busybox() {
    step "installing busybox..."
    curl -sL "$BUSYBOX_URL" -o "$ROOT/bin/busybox"
    chmod +x "$ROOT/bin/busybox"

    for applet in sh ash cat cp mv rm ls mkdir rmdir ln chmod chown \
                  grep sed awk find xargs tar gzip gunzip zstd \
                  mount umount ps kill df du free uname hostname \
                  date echo printf sleep env head tail wc sort uniq \
                  cut tr tee dd ip ping; do
        ln -sf busybox "$ROOT/bin/$applet" 2>/dev/null || true
    done
    ok "busybox ready"
}

# ─── 3. musl libc ─────────────────────────────────────────────────────────────

install_musl() {
    step "building musl libc from source..."
    local build="/tmp/musl-build"
    rm -rf "$build" && mkdir -p "$build"

    curl -sL "$MUSL_URL" | tar -xz -C "$build" --strip-components=1
    cd "$build"
    ./configure --prefix="$ROOT/usr" --syslibdir="$ROOT/lib" --silent
    make -j"$(nproc)" install >/dev/null 2>&1
    cd "$SCRIPT_DIR"

    ln -sf ../usr/lib/libc.so "$ROOT/lib/ld-musl-x86_64.so.1" 2>/dev/null || true
    rm -rf "$build"
    ok "musl ready"
}

# ─── 4. Arch package helper ───────────────────────────────────────────────────
# Downloads an Arch Linux package and extracts it into the rootfs.
# This is a BUILD-TIME operation only — pacman is never installed in the rootfs.

fetch_arch_pkg() {
    local pkgname="$1"
    local cached
    cached=$(find "$PKG_CACHE" -name "${pkgname}-*.pkg.tar.zst" 2>/dev/null | head -1)

    if [ -z "$cached" ]; then
        info "fetching $pkgname..."
        local index
        index=$(curl -s "$ARCH_MIRROR/" | grep -o "\"${pkgname}-[^\"]*\.pkg\.tar\.zst\"" | head -1 | tr -d '"')
        [ -n "$index" ] || { echo "   skip: $pkgname not found in mirror"; return 0; }
        curl -sL "$ARCH_MIRROR/$index" -o "$PKG_CACHE/$index"
        cached="$PKG_CACHE/$index"
    fi

    # Extract into rootfs, skip .PKGINFO and .MTREE metadata
    tar -I zstd -xf "$cached" -C "$ROOT" \
        --exclude='.PKGINFO' \
        --exclude='.MTREE' \
        --exclude='.BUILDINFO' \
        --exclude='.INSTALL' \
        2>/dev/null || true

    ok "$pkgname extracted"
}

# ─── 5. System packages from Arch (build-time only) ──────────────────────────

install_system_packages() {
    step "fetching system packages (build-time binary source)..."

    # Init and core
    for pkg in runit eudev dbus seatd shadow; do
        fetch_arch_pkg "$pkg"
    done

    # Audio
    for pkg in pipewire pipewire-audio wireplumber libpipewire; do
        fetch_arch_pkg "$pkg"
    done

    # Network
    for pkg in networkmanager libnm; do
        fetch_arch_pkg "$pkg"
    done

    # FessusDE stack
    for pkg in sway swaybg swaylock wlroots libwayland-client \
                waybar dunst wofi eww; do
        fetch_arch_pkg "$pkg"
    done

    # Terminal and launcher
    for pkg in kitty; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland utilities
    for pkg in wl-clipboard cliphist grim slurp wf-recorder \
                xdg-desktop-portal xdg-desktop-portal-wlr xdg-utils; do
        fetch_arch_pkg "$pkg"
    done

    # GUI apps
    for pkg in firefox thunar mousepad mpv imv \
                zathura zathura-pdf-mupdf xarchiver \
                blueman qalculate-gtk; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and icons
    for pkg in inter-font papirus-icon-theme; do
        fetch_arch_pkg "$pkg"
    done

    # Installer
    for pkg in calamares; do
        fetch_arch_pkg "$pkg"
    done

    ok "system packages installed"
}

# ─── 6. Our .myc packages ─────────────────────────────────────────────────────

install_myc_packages() {
    step "installing .myc packages via mycel-pkg..."

    for recipe in "$RECIPES"/*.myc; do
        local name
        name=$(basename "$recipe" .myc)
        info "installing $name..."
        MYCEL_ROOT="$ROOT" "$MYCEL_PKG" install "$recipe" 2>/dev/null \
            && ok "$name" || echo "   skip $name"
    done
}

# ─── 7. Base config files ─────────────────────────────────────────────────────

create_etc() {
    step "writing base configuration..."

    cat > "$ROOT/etc/os-release" <<EOF
NAME="MycelOS"
PRETTY_NAME="MycelOS"
ID=mycelos
VERSION_ID=1.0
HOME_URL=https://github.com/mycel-linux/mycel-os
SUPPORT_URL=https://github.com/mycel-linux/mycel-os/issues
EOF

    cat > "$ROOT/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
live:x:1000:1000:Live User:/home/live:/bin/sh
nobody:x:65534:65534:nobody:/:/sbin/nologin
EOF

    cat > "$ROOT/etc/group" <<'EOF'
root:x:0:root
wheel:x:10:live
audio:x:18:live
video:x:28:live
input:x:97:live
seat:x:99:live
live:x:1000:
EOF

    printf 'root:!:0:0:99999:7:::\nlive::0:0:99999:7:::\n' \
        > "$ROOT/etc/shadow"
    chmod 640 "$ROOT/etc/shadow"

    printf '127.0.0.1\tlocalhost\n127.0.1.1\tmycelos\n::1\tlocalhost\n' \
        > "$ROOT/etc/hosts"

    printf 'mycelos\n' > "$ROOT/etc/hostname"

    ok "config files written"
}

# ─── 8. runit init tree ───────────────────────────────────────────────────────

install_runit() {
    step "installing runit init tree..."

    cp -aT "$SCRIPT_DIR/../mycel-core/runit" "$ROOT/etc/sv/"

    cat > "$ROOT/etc/runit/1" <<'EOF'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t devpts devpts /dev/pts
mount -t tmpfs tmpfs /run
mkdir -p /run/runit /run/dbus /dev/shm
chmod 1777 /dev/shm
exec < /dev/console > /dev/console 2>&1
EOF

    cat > "$ROOT/etc/runit/2" <<'EOF'
#!/bin/sh
exec runsvdir /var/service
EOF

    cat > "$ROOT/etc/runit/3" <<'EOF'
#!/bin/sh
echo "MycelOS shutting down..."
EOF

    chmod +x "$ROOT/etc/runit/"{1,2,3}
    ln -sf /etc/runit/1 "$ROOT/sbin/init" 2>/dev/null || true

    mkdir -p "$ROOT/var/service"
    for svc in dbus udevd seatd pipewire wireplumber NetworkManager autologin; do
        [ -d "$ROOT/etc/sv/$svc" ] && \
            ln -sf "/etc/sv/$svc" "$ROOT/var/service/$svc" 2>/dev/null || true
    done

    ok "runit init tree ready"
}

# ─── 9. MycelOS tools ─────────────────────────────────────────────────────────

install_mycel_tools() {
    step "installing MycelOS tools..."

    install -Dm755 "$SCRIPT_DIR/../mycel/target/release/mycel"       "$ROOT/usr/bin/mycel"
    install -Dm755 "$MYCEL_PKG"                                        "$ROOT/usr/bin/mycel-pkg"
    install -Dm755 "$FESSUS_INIT"                                      "$ROOT/usr/bin/fessus-init"

    ok "mycel, mycel-pkg, fessus-init installed"
}

# ─── 10. Assets, configs, live user ──────────────────────────────────────────

install_assets_and_user() {
    step "installing assets, configs and live user..."

    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_white.png" \
        "$ROOT/usr/share/mycel/logo_white.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_black.png" \
        "$ROOT/usr/share/mycel/logo_black.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/wallpaper.jpg" \
        "$ROOT/usr/share/mycel/wallpaper.jpg"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/etc/fastfetch/config.jsonc" \
        "$ROOT/etc/fastfetch/config.jsonc"

    sed -i 's|/home/tghrl/mycelos/mycel-core/assets/||g' \
        "$ROOT/etc/fastfetch/config.jsonc"

    # Airootfs overlay (live fessus.toml, mycel.toml, sv/autologin etc.)
    cp -aT "$SCRIPT_DIR/airootfs" "$ROOT/"

    # Pre-generate FessusDE configs for the live user
    HOME="$ROOT/home/live" "$FESSUS_INIT" --apply 2>/dev/null || true

    chown -R 1000:1000 "$ROOT/home/live"

    ok "assets and live user ready"
}

# ─── 11. Squashfs ─────────────────────────────────────────────────────────────

create_squashfs() {
    step "compressing rootfs into squashfs..."
    mkdir -p "$SCRIPT_DIR/build/iso/MycelOS"

    mksquashfs "$ROOT" "$SCRIPT_DIR/build/iso/MycelOS/airootfs.sfs" \
        -comp zstd -Xcompression-level 15 -noappend -e boot \
        2>&1 | tail -3

    ok "squashfs ready ($(du -sh "$SCRIPT_DIR/build/iso/MycelOS/airootfs.sfs" | cut -f1))"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  MycelOS Bootstrap"
    echo "  ─────────────────"
    echo "  Building live rootfs from scratch."
    echo "  Arch packages used as build-time binary source only."
    echo "  Final system runs on runit + mycel-pkg — no foreign tooling."
    echo ""

    check_deps
    create_skeleton
    install_busybox
    install_musl
    create_etc
    install_system_packages
    install_myc_packages
    install_runit
    install_mycel_tools
    install_assets_and_user
    create_squashfs

    echo ""
    ok "bootstrap complete — run build.sh to create the ISO"
    echo ""
}

main "$@"
