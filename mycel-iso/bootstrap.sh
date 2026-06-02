#!/bin/bash
# MycelOS Live ISO Rootfs Bootstrap
# Builds the live filesystem from scratch.
# No base distribution is used at any point.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/build/rootfs"
RECIPES="$SCRIPT_DIR/../community/recipes"
MYCEL_PKG="$SCRIPT_DIR/../mycel-pkg/target/release/mycel-pkg"
FESSUS_INIT="$SCRIPT_DIR/../fessus/fessus-init/target/release/fessus-init"

BUSYBOX_URL="https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox"
MUSL_URL="https://musl.libc.org/releases/musl-1.2.5.tar.gz"

BLUE='\033[0;34m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'
step() { echo -e "\n${BLUE}::${NC} $1"; }
ok()   { echo -e "   ${GREEN}ok${NC} $1"; }
die()  { echo -e "   ${RED}!!${NC} $1"; exit 1; }
info() { echo -e "   ${BLUE}→${NC}  $1"; }

# ─── 0. Preflight ─────────────────────────────────────────────────────────────

check_deps() {
    step "checking build dependencies..."
    for dep in curl tar mksquashfs gcc make; do
        command -v "$dep" &>/dev/null || die "missing: $dep — install it first"
    done
    [ -x "$MYCEL_PKG" ]  || die "mycel-pkg not built — run: cd mycel-pkg && cargo build --release"
    [ -x "$FESSUS_INIT" ] || die "fessus-init not built — run: cd fessus/fessus-init && cargo build --release"
    ok "all dependencies present"
}

# ─── 1. Filesystem skeleton ────────────────────────────────────────────────────

create_skeleton() {
    step "creating filesystem skeleton..."
    rm -rf "$ROOT"

    # Standard FHS directories
    mkdir -p "$ROOT"/{bin,sbin,lib,lib64,usr/{bin,sbin,lib,lib64,share,include,local},
                     etc/{sv,runit/{1,2,3},mycel},
                     var/{lib/mycel/packages,log,run,tmp},
                     proc,sys,dev,run,tmp,
                     home,root,boot,
                     mnt,media,opt}

    chmod 1777 "$ROOT/tmp"
    chmod 0750 "$ROOT/root"

    ok "directory tree created"
}

# ─── 2. Busybox — base userland ───────────────────────────────────────────────

install_busybox() {
    step "installing busybox (base userland)..."

    curl -sL "$BUSYBOX_URL" -o "$ROOT/bin/busybox"
    chmod +x "$ROOT/bin/busybox"

    # Install all busybox applets as symlinks
    for applet in sh ash bash cat cp mv rm ls mkdir rmdir ln chmod chown \
                  grep sed awk find xargs tar gzip gunzip mount umount \
                  ps kill top df du free uname hostname date echo printf \
                  sleep which env test true false head tail wc sort uniq \
                  cut paste tr tee dd wget ip ifconfig ping; do
        ln -sf busybox "$ROOT/bin/$applet" 2>/dev/null || true
    done

    ok "busybox installed"
}

# ─── 3. musl libc — dynamic linker ────────────────────────────────────────────

install_musl() {
    step "building musl libc..."

    local build_dir="/tmp/musl-build"
    rm -rf "$build_dir"
    mkdir -p "$build_dir"

    curl -sL "$MUSL_URL" | tar -xz -C "$build_dir" --strip-components=1

    cd "$build_dir"
    ./configure \
        --prefix="$ROOT/usr" \
        --syslibdir="$ROOT/lib" \
        --disable-shared \
        --enable-static \
        --silent

    make -j"$(nproc)" install >/dev/null 2>&1
    cd "$SCRIPT_DIR"

    # Dynamic linker symlink
    ln -sf ../usr/lib/libc.so "$ROOT/lib/ld-musl-x86_64.so.1" 2>/dev/null || true

    rm -rf "$build_dir"
    ok "musl libc installed"
}

# ─── 4. Essential /etc files ──────────────────────────────────────────────────

create_etc() {
    step "writing base configuration files..."

    cat > "$ROOT/etc/os-release" <<EOF
NAME="MycelOS"
PRETTY_NAME="MycelOS"
ID=mycelos
ID_LIKE=
VERSION_ID=1.0
HOME_URL=https://github.com/mycel-linux/mycel-os
SUPPORT_URL=https://github.com/mycel-linux/mycel-os/issues
EOF

    cat > "$ROOT/etc/passwd" <<EOF
root:x:0:0:root:/root:/bin/sh
live:x:1000:1000:Live User:/home/live:/bin/sh
nobody:x:65534:65534:nobody:/:/sbin/nologin
EOF

    cat > "$ROOT/etc/group" <<EOF
root:x:0:root
wheel:x:10:live
audio:x:18:live
video:x:28:live
input:x:97:live
seat:x:99:live
live:x:1000:
EOF

    cat > "$ROOT/etc/shadow" <<EOF
root:!:0:0:99999:7:::
live::0:0:99999:7:::
EOF
    chmod 640 "$ROOT/etc/shadow"

    cat > "$ROOT/etc/hosts" <<EOF
127.0.0.1   localhost
127.0.1.1   mycelos
::1         localhost
EOF

    cat > "$ROOT/etc/hostname" <<EOF
mycelos
EOF

    cat > "$ROOT/etc/fstab" <<EOF
tmpfs   /tmp    tmpfs   defaults,nosuid,nodev   0 0
devpts  /dev/pts devpts  defaults                0 0
proc    /proc   proc    defaults                0 0
sysfs   /sys    sysfs   defaults                0 0
EOF

    ok "base config files written"
}

# ─── 5. runit init tree ───────────────────────────────────────────────────────

install_runit() {
    step "installing runit init tree..."

    # Copy our service scripts
    cp -aT "$SCRIPT_DIR/../mycel-core/runit" "$ROOT/etc/sv/"

    # runit stage 1 — mount virtual filesystems
    cat > "$ROOT/etc/runit/1" <<'EOF'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t devpts devpts /dev/pts
mount -t tmpfs tmpfs /run
mkdir -p /run/runit /run/dbus /dev/shm
chmod 1777 /dev/shm
echo "MycelOS starting..."
EOF

    # runit stage 2 — bring up services
    cat > "$ROOT/etc/runit/2" <<'EOF'
#!/bin/sh
exec runsvdir /var/service
EOF

    # runit stage 3 — clean shutdown
    cat > "$ROOT/etc/runit/3" <<'EOF'
#!/bin/sh
echo "MycelOS shutting down..."
EOF

    chmod +x "$ROOT/etc/runit/1" \
              "$ROOT/etc/runit/2" \
              "$ROOT/etc/runit/3"

    # Link init
    ln -sf /etc/runit/1 "$ROOT/sbin/init" 2>/dev/null || true

    # Enable core services
    mkdir -p "$ROOT/var/service"
    for svc in dbus udevd seatd pipewire wireplumber NetworkManager autologin; do
        [ -d "$ROOT/etc/sv/$svc" ] && \
            ln -sf "/etc/sv/$svc" "$ROOT/var/service/$svc" 2>/dev/null || true
    done

    ok "runit init tree installed"
}

# ─── 6. Install packages via mycel-pkg ────────────────────────────────────────

install_packages() {
    step "installing packages via mycel-pkg..."

    # Temporarily set ROOT for mycel-pkg installs
    export MYCEL_ROOT="$ROOT"

    for recipe in "$RECIPES"/*.myc; do
        local name
        name=$(basename "$recipe" .myc)
        info "installing $name..."
        "$MYCEL_PKG" install "$recipe" 2>/dev/null && ok "$name" || \
            echo -e "   ${RED}skip${NC} $name (check recipe)"
    done

    unset MYCEL_ROOT
}

# ─── 7. Install MycelOS tools ─────────────────────────────────────────────────

install_mycel_tools() {
    step "installing MycelOS tools..."

    install -Dm755 "$SCRIPT_DIR/../mycel/target/release/mycel" \
        "$ROOT/usr/bin/mycel"
    install -Dm755 "$MYCEL_PKG" \
        "$ROOT/usr/bin/mycel-pkg"
    install -Dm755 "$FESSUS_INIT" \
        "$ROOT/usr/bin/fessus-init"

    ok "mycel, mycel-pkg, fessus-init installed"
}

# ─── 8. Assets and configs ────────────────────────────────────────────────────

install_assets() {
    step "installing MycelOS assets and configs..."

    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_white.png" \
        "$ROOT/usr/share/mycel/logo_white.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/logo_black.png" \
        "$ROOT/usr/share/mycel/logo_black.png"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/assets/wallpaper.jpg" \
        "$ROOT/usr/share/mycel/wallpaper.jpg"
    install -Dm644 "$SCRIPT_DIR/../mycel-core/etc/fastfetch/config.jsonc" \
        "$ROOT/etc/fastfetch/config.jsonc"

    # Fix logo path in fastfetch config
    sed -i 's|/home/tghrl/mycelos/mycel-core/assets/||g' \
        "$ROOT/etc/fastfetch/config.jsonc"

    # Copy airootfs overlay (live user config, sv scripts etc.)
    cp -aT "$SCRIPT_DIR/airootfs" "$ROOT/"

    ok "assets and configs installed"
}

# ─── 9. Live user setup ───────────────────────────────────────────────────────

setup_live_user() {
    step "setting up live user..."

    mkdir -p "$ROOT/home/live/.config"
    cp -aT "$SCRIPT_DIR/airootfs/home/live/.config" \
        "$ROOT/home/live/.config/" 2>/dev/null || true

    # Generate FessusDE configs from fessus.toml
    HOME="$ROOT/home/live" \
        "$FESSUS_INIT" --apply 2>/dev/null || true

    # Set ownership (uid 1000 = live)
    chown -R 1000:1000 "$ROOT/home/live"

    ok "live user ready"
}

# ─── 10. Squashfs ─────────────────────────────────────────────────────────────

create_squashfs() {
    step "creating squashfs..."
    mkdir -p "$SCRIPT_DIR/build/iso/MycelOS"

    mksquashfs "$ROOT" "$SCRIPT_DIR/build/iso/MycelOS/airootfs.sfs" \
        -comp zstd \
        -Xcompression-level 15 \
        -noappend \
        -e boot \
        2>&1 | tail -3

    local size
    size=$(du -sh "$SCRIPT_DIR/build/iso/MycelOS/airootfs.sfs" | cut -f1)
    ok "squashfs ready ($size)"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  MycelOS Bootstrap"
    echo "  ─────────────────"
    echo "  Building live rootfs from scratch."
    echo ""

    check_deps
    create_skeleton
    install_busybox
    install_musl
    create_etc
    install_runit
    install_packages
    install_mycel_tools
    install_assets
    setup_live_user
    create_squashfs

    echo ""
    ok "bootstrap complete — run build.sh to create the ISO"
    echo ""
}

main "$@"
