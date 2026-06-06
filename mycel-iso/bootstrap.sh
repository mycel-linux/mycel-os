#!/bin/bash
# MycelOS Live ISO Rootfs Bootstrap
#
# Builds the live filesystem from scratch.
# Arch Linux packages are used as a BUILD-TIME binary source only.
# The final rootfs runs on s6 + mycel-pkg — no Arch tooling is present.
#
# Usage: bootstrap.sh [--profile <name>]
#   Profiles: fessus (default), plasma, gnome, cinnamon, xfce, budgie, mate, minimal
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/build/rootfs"
PKG_CACHE="$SCRIPT_DIR/build/pkg-cache"
RECIPES="$SCRIPT_DIR/../community/recipes"
MYCEL_PKG="$SCRIPT_DIR/../mycel-pkg/target/release/mycel-pkg"
FESSUS_INIT="$SCRIPT_DIR/../fessus/fessus-init/target/release/fessus-init"
MYCEL_COMPOSE="$SCRIPT_DIR/../mycel-compose/target/release/mycel-compose"
SERVICES="$SCRIPT_DIR/../mycel-core/services"

# ─── Profile selection ────────────────────────────────────────────────────────

# Default stubs — a profile is expected to override both of these. They are
# defined BEFORE sourcing the profile so the profile's definitions win.
install_de_packages()     { true; }
profile_desktop_section() { echo '[desktop]'; echo 'environment = "fessus"'; }

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

ARCH_MIRROR="https://geo.mirror.pkgbuild.com/extra/os/x86_64"
# Artix repos provide the non-systemd pieces Arch lacks (elogind, s6 services).
ARTIX_MIRROR="https://mirrors.rit.edu/artixlinux"

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
    [ -x "$MYCEL_COMPOSE" ] || die "mycel-compose not built — run: cd mycel-compose && cargo build --release"
    ok "dependencies ok"
}

# ─── 1. Filesystem skeleton ───────────────────────────────────────────────────

create_skeleton() {
    step "creating filesystem skeleton..."
    rm -rf "$ROOT"

    # usr-merged layout — Arch packages install everything under /usr.
    # /bin, /sbin, /lib, /lib64 are symlinks, exactly like a real Arch system.
    mkdir -p "$ROOT/usr/bin" "$ROOT/usr/lib"
    ln -s usr/bin "$ROOT/bin"
    ln -s usr/bin "$ROOT/sbin"
    ln -s usr/lib "$ROOT/lib"
    ln -s usr/lib "$ROOT/lib64"
    ln -s bin     "$ROOT/usr/sbin"
    ln -s lib     "$ROOT/usr/lib64"

    mkdir -p "$ROOT/usr/share/applications" "$ROOT/usr/share/icons"
    mkdir -p "$ROOT/usr/share/fonts" "$ROOT/usr/share/mycel"
    mkdir -p "$ROOT/usr/include" "$ROOT/usr/local"
    mkdir -p "$ROOT/etc/mycel" "$ROOT/etc/fastfetch"
    mkdir -p "$ROOT/var/lib/mycel/packages" "$ROOT/var/log" "$ROOT/var/tmp"
    mkdir -p "$ROOT/proc" "$ROOT/sys" "$ROOT/dev/pts" "$ROOT/dev/shm"
    mkdir -p "$ROOT/run" "$ROOT/tmp" "$ROOT/home/live/.config"
    mkdir -p "$ROOT/root" "$ROOT/boot" "$ROOT/mnt" "$ROOT/media" "$ROOT/opt"
    ln -s ../run "$ROOT/var/run"
    chmod 1777 "$ROOT/tmp"
    chmod 0750 "$ROOT/root"
    mkdir -p "$PKG_CACHE"
    ok "directory tree ready"
}

# ─── 2. Arch package helper ───────────────────────────────────────────────────
# Downloads an Arch Linux package and extracts it into the rootfs.
# This is a BUILD-TIME operation only — pacman is never installed in the rootfs.

# Index maps, populated by load_pkg_index():
#   PKG_FILE[name]     → "repo/filename.pkg.tar.zst"
#   PKG_DEPS[name]     → "dep1 dep2 ..."  (version constraints stripped)
#   PKG_PROVIDER[virt] → real package name that provides the virtual dep
declare -A PKG_FILE PKG_DEPS PKG_PROVIDER
declare -A FETCHED

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

    build_pkg_index
}

# Single pass over all desc files → a name/filename/deps/provides index.
build_pkg_index() {
    if [ ! -f "$PKG_CACHE/pkgindex" ]; then
        step "building package index (name → file, deps, provides)..."
        find "$PKG_CACHE/arch-db" -name desc -print0 2>/dev/null \
            | xargs -0 awk '
                function flush() {
                    if (name != "") {
                        print "P\t" name "\t" repo "/" fname "\t" deps
                        n = split(provides, pv, " ")
                        for (i = 1; i <= n; i++)
                            if (pv[i] != "") print "V\t" pv[i] "\t" name
                    }
                    name=""; fname=""; deps=""; provides=""; sec=""
                }
                FNR==1 { flush(); repo = (FILENAME ~ /\/core\//) ? "core" : "extra" }
                /^%NAME%$/     { sec="N"; next }
                /^%FILENAME%$/ { sec="F"; next }
                /^%DEPENDS%$/  { sec="D"; next }
                /^%PROVIDES%$/ { sec="P"; next }
                /^%/           { sec="";  next }
                /^$/           { next }
                {
                    if      (sec=="N") name=$0
                    else if (sec=="F") fname=$0
                    else if (sec=="D") { d=$0; gsub(/[<>=].*/,"",d); deps=deps " " d }
                    else if (sec=="P") { p=$0; gsub(/[<>=].*/,"",p); provides=provides " " p }
                }
                END { flush() }
            ' > "$PKG_CACHE/pkgindex"
        ok "indexed $(grep -c '^P' "$PKG_CACHE/pkgindex") packages"
    fi
}

PKG_INDEX_LOADED=0
load_pkg_index() {
    [ "$PKG_INDEX_LOADED" = "1" ] && return   # already loaded
    PKG_INDEX_LOADED=1
    local t a b c
    while IFS=$'\t' read -r t a b c; do
        case "$t" in
            P) PKG_FILE["$a"]="$b"; PKG_DEPS["$a"]="$c" ;;
            V) [ -z "${PKG_PROVIDER[$a]:-}" ] && PKG_PROVIDER["$a"]="$b" ;;
        esac
    done < "$PKG_CACHE/pkgindex"
}

# Download + extract a single package archive into the rootfs.
_extract_pkg() {
    local name="$1" repofile="$2"
    local repo="${repofile%%/*}" filename="${repofile##*/}"
    local cached="$PKG_CACHE/$filename"

    if [ ! -f "$cached" ]; then
        info "fetching $name..."
        curl -sL --max-time 180 --connect-timeout 15 \
            "https://geo.mirror.pkgbuild.com/${repo}/os/x86_64/${filename}" \
            -o "$cached" || { echo "   skip: download failed for $name"; rm -f "$cached"; return 0; }
    fi

    tar -I zstd -xf "$cached" -C "$ROOT" \
        --exclude='.PKGINFO' --exclude='.MTREE' \
        --exclude='.BUILDINFO' --exclude='.INSTALL' \
        2>/dev/null || true
}

# Resolve a package name (or virtual provide) and recursively fetch its full
# dependency closure before extracting it.
fetch_arch_pkg() {
    load_pkg_index

    local pkg="$1" real="$1"

    # Resolve virtual/provided names to their real package
    if [ -z "${PKG_FILE[$real]:-}" ] && [ -n "${PKG_PROVIDER[$real]:-}" ]; then
        real="${PKG_PROVIDER[$real]}"
    fi

    [ -n "${FETCHED[$real]:-}" ] && return 0
    FETCHED["$real"]=1

    local repofile="${PKG_FILE[$real]:-}"
    if [ -z "$repofile" ]; then
        echo "   skip: $pkg not in Arch repos"
        return 0
    fi

    # Fetch dependencies first (depth-first), then this package
    local d
    for d in ${PKG_DEPS[$real]:-}; do
        fetch_arch_pkg "$d"
    done

    _extract_pkg "$real" "$repofile"
}

# Fetch a single package from the Artix repos (system/world/galaxy) and extract
# it. Used for packages Arch doesn't ship (elogind, calamares). Dependencies are
# NOT resolved here — callers ensure deps come from the Arch set. Kept separate
# so Artix packages never leak into the Arch dependency index.
ARTIX_DB_READY=0
setup_artix_db() {
    [ "$ARTIX_DB_READY" = "1" ] && return
    local repo
    for repo in system world galaxy; do
        local db="$PKG_CACHE/artix-db/$repo"
        if [ ! -d "$db" ]; then
            info "downloading Artix $repo database..."
            mkdir -p "$db"
            curl -sL --max-time 120 --connect-timeout 15 \
                "$ARTIX_MIRROR/$repo/os/x86_64/$repo.db" \
                -o "$PKG_CACHE/artix-$repo.db" \
                || die "could not download Artix $repo.db"
            tar -xzf "$PKG_CACHE/artix-$repo.db" -C "$db" 2>/dev/null || true
        fi
    done
    ARTIX_DB_READY=1
}

fetch_artix_pkg() {
    local pkgname="$1"
    setup_artix_db

    # Search every Artix repo; print "repo/filename" of the matching %NAME%
    local result
    result=$(for repo in system world galaxy; do
        find "$PKG_CACHE/artix-db/$repo" -name desc -print0 2>/dev/null \
            | xargs -0 awk -v want="$pkgname" -v repo="$repo" '
                FNR==1 { name=""; f="" }
                /^%FILENAME%$/ { getline; f=$0 }
                /^%NAME%$/     { getline; name=$0 }
                name==want && f!="" { print repo "/" f; exit }
            ' 2>/dev/null
    done | head -1)
    [ -n "$result" ] || die "$pkgname not found in any Artix repo"

    local repo="${result%%/*}" fn="${result#*/}"
    local cached="$PKG_CACHE/$fn"
    if [ ! -f "$cached" ]; then
        info "fetching $pkgname from Artix ($repo)..."
        curl -sL --max-time 180 --connect-timeout 15 \
            "$ARTIX_MIRROR/$repo/os/x86_64/$fn" -o "$cached" \
            || die "could not download $pkgname from Artix"
    fi

    tar -I zstd -xf "$cached" -C "$ROOT" \
        --exclude='.PKGINFO' --exclude='.MTREE' \
        --exclude='.BUILDINFO' --exclude='.INSTALL' \
        2>/dev/null || true
    ok "$pkgname (from Artix $repo)"
}

# ─── 4b. s6 init system (built from skarnet source) ──────────────────────────
# The skarnet suite (skalibs/execline/s6/s6-rc/s6-linux-init) is NOT in the
# Arch repositories — only the AUR. Rather than depend on the AUR, we build it
# from skarnet's source tarballs. It is tiny C with no dependencies beyond a C
# compiler and builds in seconds. This is also more in keeping with MycelOS
# being an independent, source-built distribution.

SKALIBS_VER=2.15.0.0
EXECLINE_VER=2.9.9.1
S6_VER=2.15.0.0
S6RC_VER=0.6.1.1
S6LINUXINIT_VER=1.2.0.1

build_s6_suite() {
    info "building s6 supervision suite from source..."

    local b="/tmp/mycel-s6-build"
    rm -rf "$b"; mkdir -p "$b"

    # Downstream packages find skalibs/execline/etc. that we install into $ROOT
    local inc="$ROOT/usr/include"
    local lib="$ROOT/usr/lib"

    _s6_fetch() {  # name version
        local name="$1" ver="$2"
        curl -sL --max-time 120 --connect-timeout 15 \
            "https://skarnet.org/software/${name}/${name}-${ver}.tar.gz" \
            | tar -xz -C "$b" || die "could not download ${name}-${ver}"
    }

    # 1. skalibs — the base library everything else links against
    _s6_fetch skalibs "$SKALIBS_VER"
    ( cd "$b/skalibs-$SKALIBS_VER" \
        && ./configure --prefix=/usr --libdir=/usr/lib >/dev/null \
        && make -j"$(nproc)" >/dev/null \
        && make DESTDIR="$ROOT" install >/dev/null ) \
        || die "skalibs build failed"

    # 2. execline — needed at runtime by s6 and s6-rc
    _s6_fetch execline "$EXECLINE_VER"
    ( cd "$b/execline-$EXECLINE_VER" \
        && ./configure --prefix=/usr \
            --with-include="$inc" --with-lib="$lib" --with-dynlib="$lib" >/dev/null \
        && make -j"$(nproc)" >/dev/null \
        && make DESTDIR="$ROOT" install >/dev/null ) \
        || die "execline build failed"

    # 3. s6 — the supervision suite
    _s6_fetch s6 "$S6_VER"
    ( cd "$b/s6-$S6_VER" \
        && ./configure --prefix=/usr \
            --with-include="$inc" --with-lib="$lib" --with-dynlib="$lib" >/dev/null \
        && make -j"$(nproc)" >/dev/null \
        && make DESTDIR="$ROOT" install >/dev/null ) \
        || die "s6 build failed"

    # 4. s6-rc — the dependency-based service manager
    _s6_fetch s6-rc "$S6RC_VER"
    ( cd "$b/s6-rc-$S6RC_VER" \
        && ./configure --prefix=/usr \
            --with-include="$inc" --with-lib="$lib" --with-dynlib="$lib" >/dev/null \
        && make -j"$(nproc)" >/dev/null \
        && make DESTDIR="$ROOT" install >/dev/null ) \
        || die "s6-rc build failed"

    # 5. s6-linux-init — the PID 1 frontend
    _s6_fetch s6-linux-init "$S6LINUXINIT_VER"
    ( cd "$b/s6-linux-init-$S6LINUXINIT_VER" \
        && ./configure --prefix=/usr \
            --with-include="$inc" --with-lib="$lib" --with-dynlib="$lib" >/dev/null \
        && make -j"$(nproc)" >/dev/null \
        && make DESTDIR="$ROOT" install >/dev/null ) \
        || die "s6-linux-init build failed"

    rm -rf "$b"
    ok "s6 suite built and installed (skalibs, execline, s6, s6-rc, s6-linux-init)"
}

# ─── 5. System packages from Arch (build-time only) ──────────────────────────

install_system_packages() {
    step "fetching system packages (build-time binary source)..."
    setup_arch_db

    # glibc first — everything links against it
    for pkg in glibc lib32-glibc; do
        fetch_arch_pkg "$pkg"
    done

    # s6 supervision suite + PID 1 frontend (built from skarnet source)
    build_s6_suite

    # Kernel
    for pkg in linux-lts linux-firmware; do
        fetch_arch_pkg "$pkg"
    done

    # Core userland (replaces busybox with real glibc-linked tools)
    for pkg in bash coreutils grep sed gawk findutils \
                util-linux procps-ng iproute2 iputils \
                tar gzip bzip2 xz zstd file less which; do
        fetch_arch_pkg "$pkg"
    done

    # Core system. eudev is not in Arch repos, so we pull systemd purely for
    # its standalone udev (systemd-udevd + udevadm) and sysusers/tmpfiles
    # helpers. systemd is NEVER PID 1 here — s6-linux-init is.
    for pkg in systemd dbus seatd shadow pam cronie; do
        fetch_arch_pkg "$pkg"
    done

    # elogind — the standalone logind (org.freedesktop.login1). Required by
    # Plasma/GNOME for session + power management. Not in Arch; from Artix.
    # Its deps (pam, dbus, libcap, acl, util-linux) are already installed above.
    fetch_artix_pkg elogind

    # We run elogind as an s6 service, so it always owns org.freedesktop.login1
    # on the bus. Remove its dbus auto-activation file, otherwise dbus keeps
    # trying to spawn duplicates ("elogind is already running as PID ...").
    rm -f "$ROOT/usr/share/dbus-1/system-services/org.freedesktop.login1.service"

    # Basic userland tools. rsync is required by Calamares unpackfs to copy the
    # system to disk (without it the install dies with "rsync ... code 127").
    for pkg in nano curl git wget sudo polkit rsync; do
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

    # Terminal (always useful)
    for pkg in kitty; do
        fetch_arch_pkg "$pkg"
    done

    # Desktop environment — delegated to active profile
    install_de_packages

    # X11 base — only for profiles that need it
    if [ "${PROFILE_NEEDS_X11:-false}" = "true" ]; then
        info "installing Xorg base for ${PROFILE_NAME}..."
        for pkg in xorg-server xorg-xinit xorg-xrandr \
                    xf86-video-fbdev xf86-video-vesa mesa; do
            fetch_arch_pkg "$pkg"
        done
    fi

    # Installer. Calamares isn't in Arch (AUR only); Artix packages a
    # non-systemd build. Most of its heavy deps (qt6, KDE frameworks) are
    # already present from the desktop; fetch the installer-specific ones from
    # Arch, then calamares itself from Artix.
    for pkg in kpmcore parted hwinfo ckbcomp libpwquality yaml-cpp \
                polkit-qt6 qt6-location squashfs-tools \
                python python-jsonschema python-yaml \
                limine dracut efibootmgr dosfstools; do
        fetch_arch_pkg "$pkg"
    done
    fetch_artix_pkg calamares

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

# Create system users (dbus, polkitd, rtkit, seatd, ...) and the live user.
# Arch packages declare their users in /usr/lib/sysusers.d/*.conf, normally
# processed by a pacman hook (systemd-sysusers) that doesn't run in our
# extraction build — so we run it ourselves, then add the live user.
create_users() {
    step "creating system and live users..."

    # Process every package's sysusers.d to create the system accounts they need
    chroot_run systemd-sysusers 2>/dev/null || true

    # Groups the live user must belong to (create any the packages didn't)
    for g in wheel audio video input render seat bluetooth storage network lp; do
        chroot_run groupadd -rf "$g" 2>/dev/null || true
    done

    # Create the live user if sysusers/filesystem didn't
    if ! chroot_run id live >/dev/null 2>&1; then
        chroot_run useradd -m -u 1000 -s /bin/bash \
            -G wheel,audio,video,input,render,seat,bluetooth,storage,network,lp \
            live 2>/dev/null || \
        chroot_run useradd -m -u 1000 -s /bin/bash live 2>/dev/null || true
    fi

    # Passwordless root + live for the live session (login/getty need this)
    chroot_run passwd -d root 2>/dev/null || true
    chroot_run passwd -d live 2>/dev/null || true

    ok "users ready"
}

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

    # /etc/shells — required by chsh, useradd and login
    cat > "$ROOT/etc/shells" <<'EOF'
/bin/sh
/bin/bash
/usr/bin/bash
EOF

    # Minimal fstab — the live system runs from squashfs; the installer writes
    # the real fstab. tmpfs entries keep /tmp and /dev/shm sane.
    cat > "$ROOT/etc/fstab" <<'EOF'
tmpfs /tmp     tmpfs nosuid,nodev,mode=1777 0 0
tmpfs /dev/shm tmpfs nosuid,nodev,mode=1777 0 0
EOF

    printf '127.0.0.1\tlocalhost\n127.0.1.1\tmycelos\n::1\tlocalhost\n' \
        > "$ROOT/etc/hosts"
    printf 'mycelos\n' > "$ROOT/etc/hostname"

    # nsswitch.conf — without it, getpwnam/getgrnam lookups misbehave
    cat > "$ROOT/etc/nsswitch.conf" <<'EOF'
passwd: files
group: files
shadow: files
hosts: files dns
networks: files
protocols: files
services: files
ethers: files
rpc: files
EOF

    # sudo for the wheel group, passwordless on the live system so the
    # installer (and the live user) can elevate without a password prompt.
    mkdir -p "$ROOT/etc/sudoers.d"
    echo '%wheel ALL=(ALL:ALL) NOPASSWD: ALL' > "$ROOT/etc/sudoers.d/10-wheel"
    chmod 0440 "$ROOT/etc/sudoers.d/10-wheel"

    # polkit: let the wheel group run Calamares without authentication, so the
    # installer launches straight from the live Plasma menu.
    mkdir -p "$ROOT/etc/polkit-1/rules.d"
    cat > "$ROOT/etc/polkit-1/rules.d/49-mycel-live.rules" <<'EOF'
// Live-session convenience: wheel runs the installer + admin actions freely.
polkit.addRule(function(action, subject) {
    if (subject.isInGroup("wheel")) {
        if (action.id.indexOf("com.github.calamares") === 0 ||
            action.id.indexOf("org.freedesktop.udisks2") === 0 ||
            action.id.indexOf("org.freedesktop.login1") === 0) {
            return polkit.Result.YES;
        }
    }
});
EOF

    # PAM stack for login. Self-contained (no fragile include chain). The key
    # line is `session ... pam_elogind.so`, which registers a logind session
    # and exports XDG_RUNTIME_DIR — required for Plasma/GNOME under elogind.
    mkdir -p "$ROOT/etc/pam.d"
    cat > "$ROOT/etc/pam.d/login" <<'EOF'
#%PAM-1.0
auth      sufficient pam_unix.so   nullok
auth      required   pam_deny.so
account   required   pam_unix.so
session   required   pam_unix.so
session   optional   pam_loginuid.so
session   optional   pam_elogind.so
EOF
    # A sane fallback for any other PAM-using service
    cat > "$ROOT/etc/pam.d/other" <<'EOF'
#%PAM-1.0
auth      required pam_unix.so
account   required pam_unix.so
session   required pam_unix.so
password  required pam_unix.so
EOF

    # Desktop session launcher. Sourced by every login shell via /etc/profile;
    # on tty1 it reads [desktop] environment from mycel.toml and execs the right
    # session. pam_elogind has already set XDG_RUNTIME_DIR by this point.
    cat > "$ROOT/etc/profile.d/zz-mycel-session.sh" <<'EOF'
# Start the graphical session automatically on tty1 (autologin)
if [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${DISPLAY:-}" ] && [ "$(tty)" = "/dev/tty1" ]; then
    [ -z "${XDG_RUNTIME_DIR:-}" ] && export XDG_RUNTIME_DIR="/run/user/$(id -u)"
    [ -d "$XDG_RUNTIME_DIR" ] || { mkdir -p "$XDG_RUNTIME_DIR"; chmod 700 "$XDG_RUNTIME_DIR"; }
    export XDG_SESSION_TYPE=wayland
    _de=$(awk -F'"' '/^\[desktop\]/{f=1} f&&/^environment/{print $2;exit}' /etc/mycel.toml 2>/dev/null)
    case "${_de:-plasma}" in
        plasma)   export XDG_CURRENT_DESKTOP=KDE;          exec dbus-run-session startplasma-wayland ;;
        gnome)    export XDG_CURRENT_DESKTOP=GNOME;        exec dbus-run-session gnome-session ;;
        budgie)   export XDG_CURRENT_DESKTOP=Budgie:GNOME; exec dbus-run-session budgie-session ;;
        hyprland) export XDG_CURRENT_DESKTOP=Hyprland;     exec dbus-run-session Hyprland ;;
        sway|fessus) export XDG_CURRENT_DESKTOP=sway;      exec dbus-run-session sway ;;
        cinnamon) unset XDG_SESSION_TYPE; exec startx /usr/bin/cinnamon-session ;;
        xfce)     unset XDG_SESSION_TYPE; exec startx /usr/bin/startxfce4 ;;
        mate)     unset XDG_SESSION_TYPE; exec startx /usr/bin/mate-session ;;
        none)     : ;;   # stay at the shell
        *)        export XDG_CURRENT_DESKTOP=KDE;          exec dbus-run-session startplasma-wayland ;;
    esac
fi
EOF

    ok "config files written"
}

# ─── Helper: run a command inside the rootfs chroot ──────────────────────────

chroot_unmount() {
    umount "$ROOT/dev/pts" 2>/dev/null || true
    umount "$ROOT/dev"     2>/dev/null || true
    umount "$ROOT/sys"     2>/dev/null || true
    umount "$ROOT/proc"    2>/dev/null || true
}

chroot_run() {
    local rc=0

    mkdir -p "$ROOT/proc" "$ROOT/sys" "$ROOT/dev" "$ROOT/dev/pts"
    mount -o bind /proc    "$ROOT/proc"
    mount -o bind /sys     "$ROOT/sys"
    mount -o bind /dev     "$ROOT/dev"
    mount -o bind /dev/pts "$ROOT/dev/pts"

    # Ensure mounts are always cleaned up even if chroot fails under set -e
    trap chroot_unmount RETURN

    # Capture exit code without `local` (which would clobber $?)
    chroot "$ROOT" "$@" && rc=0 || rc=$?

    chroot_unmount
    trap - RETURN
    return $rc
}

# ─── 7b. Post-install hooks ──────────────────────────────────────────────────
# We extract packages directly, so the pacman install hooks that normally make
# a desktop functional never run. Replicate the important ones in the chroot:
# ld.so cache, GSettings schema compilation (without it GTK apps and portals
# abort with SIGABRT), icon/font/mime/desktop caches.

run_post_install_hooks() {
    step "running post-install hooks (ldconfig, schemas, caches)..."

    # Hide iconless/clutter launchers pulled in as dependencies that don't
    # belong in MycelOS (we manage software with mycel-pkg, not a GUI store).
    for d in avahi-discover bssh bvnc qv4l2 qvidcap; do
        f="$ROOT/usr/share/applications/${d}.desktop"
        [ -f "$f" ] && echo "NoDisplay=true" >> "$f"
    done

    chroot_run ldconfig 2>/dev/null || true

    [ -d "$ROOT/usr/share/glib-2.0/schemas" ] && \
        chroot_run glib-compile-schemas /usr/share/glib-2.0/schemas 2>/dev/null || true

    [ -d "$ROOT/usr/share/applications" ] && \
        chroot_run update-desktop-database /usr/share/applications 2>/dev/null || true

    [ -d "$ROOT/usr/share/mime" ] && \
        chroot_run update-mime-database /usr/share/mime 2>/dev/null || true

    for theme in hicolor breeze breeze-dark Papirus Papirus-Dark Adwaita; do
        [ -d "$ROOT/usr/share/icons/$theme" ] && \
            chroot_run gtk-update-icon-cache -qtf "/usr/share/icons/$theme" 2>/dev/null || true
    done

    chroot_run fc-cache -f 2>/dev/null || true

    ok "post-install hooks done"
}

# ─── 8. s6 init tree ─────────────────────────────────────────────────────────

install_s6_tree() {
    step "installing s6 init tree..."

    # Weave the s6-rc service source tree from declarative definitions.
    # mycel-compose reads one .toml per service from mycel-core/services and
    # emits the full s6-rc source tree (run scripts, type, dependencies,
    # bundle contents) — the declarative service layer that makes stitching
    # services together a matter of editing a .toml, not hand-rolling glue.
    mkdir -p "$ROOT/etc/s6-rc"
    "$MYCEL_COMPOSE" --services "$SERVICES" --out "$ROOT/etc/s6-rc/source" \
        || die "mycel-compose failed to generate the s6-rc source tree"

    # Compile the s6-rc database inside the rootfs (needs the s6-rc binary)
    chroot_run s6-rc-compile /etc/s6-rc/compiled /etc/s6-rc/source \
        || die "s6-rc-compile failed — is s6-rc installed in the rootfs?"

    # Set up s6-linux-init as PID 1. Flags per the s6-linux-init-maker docs:
    #   -1                 send stage-1 messages to /dev/console (visible logs)
    #   -B                 run without a catch-all logger (we have no log user)
    #   -p <path>          initial PATH for PID 1
    #   -c <basedir>       where the config lives at boot time
    #   final positional   directory to generate the config into now
    # The generated init runs /etc/s6-linux-init/scripts/rc.init as stage 2,
    # with s6-svscan already supervising the scandir at /run/service.
    chroot_run s6-linux-init-maker \
        -1 -B \
        -p /usr/bin:/usr/sbin:/bin:/sbin \
        -c /etc/s6-linux-init \
        /etc/s6-linux-init \
        || die "s6-linux-init-maker failed"

    # Install our stage 2 / shutdown scripts over the generated defaults
    install -Dm755 "$SCRIPT_DIR/../mycel-core/s6-linux-init/scripts/rc.init" \
        "$ROOT/etc/s6-linux-init/scripts/rc.init"
    install -Dm755 "$SCRIPT_DIR/../mycel-core/s6-linux-init/scripts/rc.shutdown" \
        "$ROOT/etc/s6-linux-init/scripts/rc.shutdown"

    # /sbin/init → s6-linux-init's init. MUST be a RELATIVE symlink: dracut
    # validates init with `[ -x /sysroot/sbin/init ]` before switch_root, and an
    # absolute symlink target resolves against the initramfs root (where it does
    # not exist) → "Cannot find init". /sbin and /usr/sbin are usr-merged to
    # /usr/bin, so we write the link there. From /usr/bin, /etc is ../../etc.
    ln -sfn ../../etc/s6-linux-init/bin/init "$ROOT/usr/bin/init"

    # Expose halt / poweroff / reboot / shutdown in PATH (relative links too).
    for cmd in halt poweroff reboot shutdown telinit; do
        [ -e "$ROOT/etc/s6-linux-init/bin/$cmd" ] && \
            ln -sfn "../../etc/s6-linux-init/bin/$cmd" "$ROOT/usr/bin/$cmd"
    done

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

    # Seed a record for every Arch package whose files actually landed in the
    # rootfs. We read the cached .pkg.tar.zst files (one per installed package)
    # rather than the full repo DB, so we only record what we really installed.
    local pkgfile pkgname pkgver
    for pkgfile in "$PKG_CACHE"/*.pkg.tar.zst; do
        [ -e "$pkgfile" ] || continue

        # .PKGINFO holds pkgname and pkgver — read it straight from the archive
        local pkginfo
        pkginfo=$(tar -I zstd -xOf "$pkgfile" .PKGINFO 2>/dev/null || true)
        [ -n "$pkginfo" ] || continue

        pkgname=$(printf '%s\n' "$pkginfo" | awk -F' = ' '/^pkgname/{print $2; exit}')
        pkgver=$(printf  '%s\n' "$pkginfo" | awk -F' = ' '/^pkgver/{print $2; exit}')

        [ -n "$pkgname" ] || continue
        [ -n "$pkgver" ]  || pkgver="0.0.0"

        cat > "$db/${pkgname}.toml" <<TOML
name         = "${pkgname}"
version      = "${pkgver}"
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
    install -Dm755 "$MYCEL_COMPOSE"                                    "$ROOT/usr/bin/mycel-compose"

    # Install the service declarations so the running system can recompose its
    # s6-rc database (mycel switch → mycel-compose → s6-rc-update). These are the
    # source of truth; editing one and running `mycel switch` applies it live.
    mkdir -p "$ROOT/etc/mycel/services"
    cp -a "$SERVICES"/*.toml "$ROOT/etc/mycel/services/"

    ok "mycel, mycel-pkg, fessus-init, mycel-compose installed"
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

    # Patch the live mycel.toml with the profile's desktop + services sections
    local mycel_toml="$ROOT/etc/mycel.toml"
    # Strip existing [desktop] and [services] sections, then append profile ones
    awk '
        /^\[(desktop|services)\]/ { skip=1 }
        /^\[/ && !/^\[(desktop|services)\]/ { skip=0 }
        !skip { print }
    ' "$mycel_toml" > "$mycel_toml.tmp"
    printf '\n' >> "$mycel_toml.tmp"
    profile_desktop_section >> "$mycel_toml.tmp"
    mv "$mycel_toml.tmp" "$mycel_toml"

    # Pre-generate desktop configs for the live user (fessus only)
    if [ "${PROFILE_ENV}" = "fessus" ] || [ "${PROFILE_ENV}" = "hyprland" ]; then
        HOME="$ROOT/home/live" "$FESSUS_INIT" --apply 2>/dev/null || true
    fi

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
    install_system_packages
    # users + base config AFTER packages, so the filesystem package's /etc
    # files don't clobber ours and sysusers can see every package's declarations
    create_users
    create_etc
    run_post_install_hooks
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
