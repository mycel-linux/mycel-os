#!/bin/bash
# MycelOS Live ISO Rootfs Bootstrap
#
# Builds the live filesystem from scratch.
# Arch Linux packages are used as a BUILD-TIME binary source only.
# The final rootfs runs on s6 + mycel-pkg — no Arch tooling is present.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/build/rootfs"
PKG_CACHE="$SCRIPT_DIR/build/pkg-cache"
RECIPES="$SCRIPT_DIR/../community/recipes"
MYCEL_PKG="$SCRIPT_DIR/../mycel-pkg/target/release/mycel-pkg"
FESSUS_INIT="$SCRIPT_DIR/../fessus/fessus-init/target/release/fessus-init"

ARCH_MIRROR="https://geo.mirror.pkgbuild.com/extra/os/x86_64"

BLUE='\033[0;34m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
step() { echo -e "\n${BLUE}::${NC} $1"; }
ok()   { echo -e "   ${GREEN}ok${NC} $1"; }
die()  { echo -e "   ${RED}!!${NC} $1"; exit 1; }
info() { echo -e "   → $1"; }

# ─── Preflight ────────────────────────────────────────────────────────────────

check_deps() {
    step "checking build dependencies..."
    for dep in curl tar zstd mksquashfs; do
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
    mkdir -p "$ROOT/bin" "$ROOT/sbin" "$ROOT/lib" "$ROOT/lib64"
    mkdir -p "$ROOT/usr/bin" "$ROOT/usr/sbin" "$ROOT/usr/lib" "$ROOT/usr/lib64"
    mkdir -p "$ROOT/usr/share/applications" "$ROOT/usr/share/icons"
    mkdir -p "$ROOT/usr/share/fonts" "$ROOT/usr/share/mycel"
    mkdir -p "$ROOT/usr/include" "$ROOT/usr/local"
    mkdir -p "$ROOT/etc/s6/sv"
    mkdir -p "$ROOT/etc/mycel" "$ROOT/etc/fastfetch" "$ROOT/etc/sway"
    mkdir -p "$ROOT/etc/waybar" "$ROOT/etc/dunst"
    mkdir -p "$ROOT/var/lib/mycel/packages" "$ROOT/var/log" "$ROOT/var/run" "$ROOT/var/tmp"
    mkdir -p "$ROOT/proc" "$ROOT/sys" "$ROOT/dev/pts" "$ROOT/dev/shm"
    mkdir -p "$ROOT/run" "$ROOT/tmp" "$ROOT/home/live/.config"
    mkdir -p "$ROOT/root" "$ROOT/boot" "$ROOT/mnt" "$ROOT/media" "$ROOT/opt"
    chmod 1777 "$ROOT/tmp"
    chmod 0750 "$ROOT/root"
    mkdir -p "$PKG_CACHE"
    ok "directory tree ready"
}

# ─── 2. Arch package helper ───────────────────────────────────────────────────
# Downloads an Arch Linux package and extracts it into the rootfs.
# This is a BUILD-TIME operation only — pacman is never installed in the rootfs.

setup_arch_db() {
    if [ ! -d "$PKG_CACHE/arch-db" ]; then
        step "downloading Arch package database..."
        mkdir -p "$PKG_CACHE/arch-db/extra" "$PKG_CACHE/arch-db/core"

        curl -sL --max-time 120 --connect-timeout 15 \
            "https://geo.mirror.pkgbuild.com/extra/os/x86_64/extra.db" \
            -o "$PKG_CACHE/extra.db" || die "could not download Arch extra.db"

        curl -sL --max-time 60 --connect-timeout 15 \
            "https://geo.mirror.pkgbuild.com/core/os/x86_64/core.db" \
            -o "$PKG_CACHE/core.db" || die "could not download Arch core.db"

        tar -xzf "$PKG_CACHE/extra.db" -C "$PKG_CACHE/arch-db/extra" 2>/dev/null || true
        tar -xzf "$PKG_CACHE/core.db"  -C "$PKG_CACHE/arch-db/core"  2>/dev/null || true

        ok "package database ready"
    fi
}

find_arch_pkg_filename() {
    local pkgname="$1"
    local desc_file

    for repo in extra core; do
        desc_file=$(find "$PKG_CACHE/arch-db/$repo" -name "desc" 2>/dev/null \
            | xargs grep -l "^${pkgname}$" 2>/dev/null | head -1)
        if [ -n "$desc_file" ]; then
            local repo_found="$repo"
            local filename
            filename=$(awk '/^%FILENAME%/{getline; print}' "$desc_file")
            echo "$repo_found/$filename"
            return 0
        fi
    done
    return 1
}

fetch_arch_pkg() {
    local pkgname="$1"
    local cached
    cached=$(find "$PKG_CACHE" -name "${pkgname}-*.pkg.tar.zst" 2>/dev/null | head -1)

    if [ -z "$cached" ]; then
        local result
        result=$(find_arch_pkg_filename "$pkgname") || {
            echo "   skip: $pkgname not in Arch repos"
            return 0
        }

        local repo filename
        repo=$(dirname "$result")
        filename=$(basename "$result")

        info "fetching $pkgname..."
        curl -sL --max-time 120 --connect-timeout 15 \
            "https://geo.mirror.pkgbuild.com/${repo}/os/x86_64/${filename}" \
            -o "$PKG_CACHE/$filename" || {
            echo "   skip: download failed for $pkgname"
            return 0
        }
        cached="$PKG_CACHE/$filename"
    fi

    tar -I zstd -xf "$cached" -C "$ROOT" \
        --exclude='.PKGINFO' \
        --exclude='.MTREE' \
        --exclude='.BUILDINFO' \
        --exclude='.INSTALL' \
        2>/dev/null || true

    ok "$pkgname"
}

# ─── 4b. s6 init system ───────────────────────────────────────────────────────

install_s6() {
    info "installing s6 supervision suite..."
    for pkg in skalibs execline s6 s6-rc s6-linux-init; do
        fetch_arch_pkg "$pkg"
    done
    ok "s6 installed"
}

# ─── 5. System packages from Arch (build-time only) ──────────────────────────

install_system_packages() {
    step "fetching system packages (build-time binary source)..."
    setup_arch_db

    # s6 supervision suite + PID 1 frontend
    install_s6

    # Kernel
    for pkg in linux-lts linux-firmware; do
        fetch_arch_pkg "$pkg"
    done

    # glibc — the one and only C runtime
    for pkg in glibc lib32-glibc; do
        fetch_arch_pkg "$pkg"
    done

    # Core userland (replaces busybox with real glibc-linked tools)
    for pkg in bash coreutils grep sed gawk findutils \
                util-linux procps-ng iproute2 iputils \
                tar gzip bzip2 xz zstd file less which; do
        fetch_arch_pkg "$pkg"
    done

    # Core system
    for pkg in eudev dbus seatd shadow pam; do
        fetch_arch_pkg "$pkg"
    done

    # Basic userland tools
    for pkg in nano curl git wget; do
        fetch_arch_pkg "$pkg"
    done

    # Bluetooth
    for pkg in bluez bluez-utils; do
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

    # FessusDE stack (sway)
    for pkg in sway swaybg swaylock wlroots libwayland-client \
                waybar dunst wofi eww; do
        fetch_arch_pkg "$pkg"
    done

    # Hyprland stack
    for pkg in hyprland hyprpaper hyprlock hypridle \
                xdg-desktop-portal-hyprland; do
        fetch_arch_pkg "$pkg"
    done

    # Terminal
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
bluetooth:x:85:live
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

# ─── Helper: run a command inside the rootfs chroot ──────────────────────────

chroot_run() {
    mount -o bind /proc    "$ROOT/proc"
    mount -o bind /sys     "$ROOT/sys"
    mount -o bind /dev     "$ROOT/dev"
    mount -o bind /dev/pts "$ROOT/dev/pts"

    chroot "$ROOT" "$@"
    local rc=$?

    umount "$ROOT/dev/pts" 2>/dev/null || true
    umount "$ROOT/dev"     2>/dev/null || true
    umount "$ROOT/sys"     2>/dev/null || true
    umount "$ROOT/proc"    2>/dev/null || true

    return $rc
}

# ─── 8. s6 init tree ─────────────────────────────────────────────────────────

install_s6_tree() {
    step "installing s6 init tree..."

    # Copy s6-rc service source definitions
    mkdir -p "$ROOT/etc/s6-rc"
    cp -aT "$SCRIPT_DIR/../mycel-core/s6-rc/source" "$ROOT/etc/s6-rc/source"

    # Compile the s6-rc database inside the rootfs (needs the s6-rc binary)
    chroot_run s6-rc-compile /etc/s6-rc/compiled /etc/s6-rc/source \
        || die "s6-rc-compile failed — is s6-rc installed in the rootfs?"

    # Set up s6-linux-init as PID 1
    # s6-linux-init-maker generates the init tree at /etc/s6-linux-init
    chroot_run s6-linux-init-maker \
        -c /etc/s6-linux-init \
        -s /run/s6-linux-init \
        -b /usr/bin \
        -u nobody \
        -L \
        /etc/s6-linux-init \
        || die "s6-linux-init-maker failed"

    # Install our stage 2 / shutdown scripts over the defaults
    install -Dm755 "$SCRIPT_DIR/../mycel-core/s6-linux-init/scripts/rc.init" \
        "$ROOT/etc/s6-linux-init/scripts/rc.init"
    install -Dm755 "$SCRIPT_DIR/../mycel-core/s6-linux-init/scripts/rc.shutdown" \
        "$ROOT/etc/s6-linux-init/scripts/rc.shutdown"

    # /sbin/init → the generated s6-linux-init binary
    ln -sf /etc/s6-linux-init/init "$ROOT/sbin/init"

    ok "s6 init tree ready"
}

# ─── 9. Seed mycel-pkg package database ──────────────────────────────────────
# Writes a minimal .toml record for every Arch package that bootstrap installed
# so that `mycel switch` on first boot sees them as already present and doesn't
# try to reinstall everything.

seed_package_db() {
    step "seeding mycel-pkg package database..."

    local db="$ROOT/var/lib/mycel/packages"
    mkdir -p "$db"

    local now
    now=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # Walk the Arch package DB desc files we downloaded during build
    find "$PKG_CACHE/arch-db" -name "desc" | while read -r desc; do
        local name version
        name=$(awk '/^%NAME%/{getline; print; exit}' "$desc")
        version=$(awk '/^%VERSION%/{getline; print; exit}' "$desc")

        [ -z "$name" ] || [ -z "$version" ] && continue

        # Only seed packages that are actually present in the rootfs
        [ -f "$ROOT/usr/bin/$name" ] || \
        [ -f "$ROOT/usr/sbin/$name" ] || \
        [ -f "$ROOT/usr/lib/$name" ] || \
        [ -d "$ROOT/usr/share/$name" ] || \
        [ -f "$ROOT/usr/bin/${name}d" ] || \
        grep -qx "$name" "$PKG_CACHE/installed.list" 2>/dev/null \
            || continue

        cat > "$db/${name}.toml" <<TOML
name         = "${name}"
version      = "${version}"
installed_at = "${now}"

[files]
installed = []
TOML
    done

    # Also seed from an explicit list of packages we know we installed
    local installed=(
        bash coreutils grep sed gawk findutils util-linux procps-ng
        iproute2 iputils tar gzip bzip2 xz zstd file less which
        glibc eudev dbus seatd shadow pam
        skalibs execline s6 s6-rc s6-linux-init
        linux-lts linux-firmware
        nano curl git wget
        bluez bluez-utils
        pipewire pipewire-audio wireplumber
        networkmanager
        sway swaybg swaylock wlroots waybar dunst wofi kitty
        wl-clipboard grim slurp cliphist wf-recorder
        xdg-desktop-portal xdg-utils
        firefox thunar mousepad mpv imv
        zathura xarchiver blueman qalculate-gtk
        inter-font papirus-icon-theme
        calamares
    )

    for pkg in "${installed[@]}"; do
        local record="$db/${pkg}.toml"
        [ -f "$record" ] && continue   # already seeded from arch-db

        # Try to find the version in the arch-db
        local version="0.0.0"
        local desc_file
        desc_file=$(find "$PKG_CACHE/arch-db" -name "desc" 2>/dev/null \
            | xargs grep -l "^${pkg}$" 2>/dev/null | head -1)
        if [ -n "$desc_file" ]; then
            version=$(awk '/^%VERSION%/{getline; print; exit}' "$desc_file")
        fi

        cat > "$record" <<TOML
name         = "${pkg}"
version      = "${version:-0.0.0}"
installed_at = "${now}"

[files]
installed = []
TOML
    done

    local count
    count=$(find "$db" -name "*.toml" | wc -l)
    ok "seeded ${count} packages"
}

# ─── 11. /etc/skel — default files for new user home directories ─────────────

install_skel() {
    step "installing /etc/skel..."

    mkdir -p "$ROOT/etc/skel/.config"

    cat > "$ROOT/etc/skel/.bashrc" <<'EOF'
# ~/.bashrc — sourced for interactive non-login shells

# If not interactive, do nothing
[[ $- != *i* ]] && return

# Prompt
PS1='\[\e[1;34m\]\u@\h\[\e[0m\]:\[\e[1;36m\]\w\[\e[0m\]\$ '

# Aliases
alias ls='ls --color=auto'
alias ll='ls -lah --color=auto'
alias la='ls -A --color=auto'
alias grep='grep --color=auto'
alias diff='diff --color=auto'

# Handy shortcuts
alias mycel-log='journalctl -xe 2>/dev/null || tail -f /var/log/messages'

# Load zoxide if installed
command -v zoxide >/dev/null 2>&1 && eval "$(zoxide init bash)"

# Load starship if installed
command -v starship >/dev/null 2>&1 && eval "$(starship init bash)"
EOF

    cat > "$ROOT/etc/skel/.bash_profile" <<'EOF'
# ~/.bash_profile — sourced for login shells
[[ -f ~/.bashrc ]] && source ~/.bashrc
EOF

    cat > "$ROOT/etc/skel/.profile" <<'EOF'
# ~/.profile — POSIX-compatible login shell config
export PATH="$HOME/.local/bin:$PATH"
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_CACHE_HOME="$HOME/.cache"
EOF

    # Copy the live fessus.toml as the default desktop config for new users
    if [ -f "$SCRIPT_DIR/airootfs/home/live/.config/fessus.toml" ]; then
        cp "$SCRIPT_DIR/airootfs/home/live/.config/fessus.toml" \
            "$ROOT/etc/skel/.config/fessus.toml"
    fi

    ok "/etc/skel ready"
}

# ─── 12. MycelOS tools ────────────────────────────────────────────────────────

install_mycel_tools() {
    step "installing MycelOS tools..."

    install -Dm755 "$SCRIPT_DIR/../mycel/target/release/mycel"       "$ROOT/usr/bin/mycel"
    install -Dm755 "$MYCEL_PKG"                                        "$ROOT/usr/bin/mycel-pkg"
    install -Dm755 "$FESSUS_INIT"                                      "$ROOT/usr/bin/fessus-init"

    ok "mycel, mycel-pkg, fessus-init installed"
}

# ─── 13. Assets, configs, live user ──────────────────────────────────────────

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

    # Airootfs overlay (live fessus.toml, mycel.toml, s6/autologin etc.)
    cp -aT "$SCRIPT_DIR/airootfs" "$ROOT/"

    # Pre-generate FessusDE configs for the live user
    HOME="$ROOT/home/live" "$FESSUS_INIT" --apply 2>/dev/null || true

    chown -R 1000:1000 "$ROOT/home/live"

    ok "assets and live user ready"
}

# ─── 11. Calamares installer config ──────────────────────────────────────────

install_calamares_config() {
    step "installing Calamares installer configuration..."

    local cal_etc="$ROOT/etc/calamares"
    mkdir -p "$cal_etc/modules" "$cal_etc/branding"

    cp "$SCRIPT_DIR/../mycel-installer/settings.conf" "$cal_etc/"

    for conf in "$SCRIPT_DIR/../mycel-installer/module-configs/"*.conf; do
        cp "$conf" "$cal_etc/modules/"
    done

    cp -aT "$SCRIPT_DIR/../mycel-installer/branding/mycel" \
        "$cal_etc/branding/mycel"

    # Custom modules go where Calamares looks for Python job modules
    local mod_dir="$ROOT/usr/lib/calamares/modules"
    mkdir -p "$mod_dir"
    for mod in "$SCRIPT_DIR/../mycel-installer/modules/"*/; do
        cp -aT "$mod" "$mod_dir/$(basename "$mod")"
    done

    ok "Calamares config installed"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  MycelOS Bootstrap"
    echo "  ─────────────────"
    echo "  Building live rootfs from scratch."
    echo "  Arch packages used as build-time binary source only."
    echo "  Final system runs on s6 + mycel-pkg — no foreign tooling."
    echo ""

    check_deps
    create_skeleton
    create_etc
    install_system_packages
    install_myc_packages
    install_s6_tree
    install_skel
    seed_package_db
    install_mycel_tools
    install_assets_and_user
    install_calamares_config

    echo ""
    ok "bootstrap complete — run build.sh to create the ISO"
    echo ""
}

main "$@"
