<div align="center">
  <img src="mycel-core/assets/logo_black.png" width="120" />
  <h1>MycelOS</h1>
  <p><strong>The First Rhizomatic OS</strong></p>
  <p>Declarative · Immutable · Independent · s6</p>

  ![License](https://img.shields.io/badge/license-GPL--3.0-blue)
  ![Status](https://img.shields.io/badge/status-boots%20to%20plasma-brightgreen)
  ![Init](https://img.shields.io/badge/init-s6-purple)
  ![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey)
</div>

---

MycelOS is an independent Linux distribution built from scratch. Not a fork. Not based on anything. It runs on **s6** — a modern, actively maintained process supervision suite — driven by a single declarative file (`mycel.toml`), and boots to a full **KDE Plasma** desktop.

No systemd. No runit. No OpenRC. The C runtime is glibc, the init is s6-linux-init as PID 1, services are dependency-ordered by s6-rc, and logind is provided by elogind — a complete, modern, systemd-free desktop stack.

## Rhizomatic

A rhizome is the underground network a mycelium grows from — no center, no trunk, no hierarchy; every node connects directly to every other. MycelOS is built the same way, and "rhizomatic" isn't decoration — it's the architecture:

- **Decentralized packages** — software comes from a web of GitHub overlays (`github:anyone/their-packages`), not one blessed central repository.
- **Declarative** — you describe the whole organism in `mycel.toml` and it grows into that shape, instead of issuing an imperative chain of commands.
- **A graph, not a hierarchy** — s6-rc resolves services as a network of dependencies; there's no central daemon that owns the system.
- **Immutable & branching** — every `mycel switch` is a new generation; old ones are sealed read-only and you can roll back to any of them. Prune a branch, the network survives.

NixOS is declarative but centralized — one package tree, one builder. No one has claimed *rhizomatic* as an identity. MycelOS does.

## Why s6?

The anti-systemd space is crowded but stagnant. Void and Artix run runit — a supervision suite last seriously updated in 2004. Alpine and Gentoo run OpenRC — a service manager, not a real supervisor. Respected, but old.

s6, written by Laurent Bercot and actively maintained, is a proper process supervision suite: clean C, real dependency handling via s6-rc, and a design with none of the accumulated debt. MycelOS is one of the first independent distributions built ground-up on s6 — and built genuinely from scratch: the entire userland is assembled from upstream binary packages and source, with the skarnet suite (skalibs/execline/s6/s6-rc/s6-linux-init) compiled from source at build time.

## The stack

| Component | Choice | Why |
|---|---|---|
| Init | **s6-linux-init + s6-rc** | Modern, actively maintained, dependency-ordered supervision; clean shutdown/reboot |
| Login/seat | **elogind + seatd** | Standalone logind (`login1`) + seat management, no systemd |
| C runtime | **glibc** | The one and only libc — full binary compatibility |
| Desktop | **KDE Plasma 6** | Stable, polished, Wayland. Other DEs available as separate editions |
| Packages | **mycel-pkg + .myc** | Own format, GitHub-native, no foreign tooling on the installed system |
| Config | **mycel.toml** | One file declares your entire system |
| Installer | **Calamares** | Offline GUI install — copies the live squashfs, no network needed |
| Bootloader | **Limine** | Modern, fast, BIOS + UEFI |

## mycel.toml

One file. Your whole system.

```toml
[system]
hostname = "mycelbox"
timezone = "America/New_York"
locale   = "en_US.UTF-8"
kernel   = "performance"
channel  = "stable"

[packages]
install = ["firefox", "kitty", "git", "btop", "neovim"]
lock    = ["firefox"]

[overlays]
sources = ["github:mycel-linux/community"]

[desktop]
environment = "plasma"

[services]
enable = ["pipewire", "wireplumber", "NetworkManager", "bluetooth"]

[[users]]
name   = "alice"
shell  = "bash"
groups = ["wheel", "audio", "video", "input", "seat"]
password_hash = ""
```

Run `mycel switch` to apply. Every change creates a new generation you can roll back to.

## Desktops

MycelOS ships **one ISO per desktop** (the Artix model) — pick your edition, download only what you need, install fully offline. KDE Plasma is the flagship.

| Edition | Desktop | Session |
|---|---|---|
| **plasma** (default) | KDE Plasma 6 | Wayland |
| gnome | GNOME | Wayland |
| cinnamon | Cinnamon | X11 |
| xfce | XFCE | X11 |
| budgie | Budgie | Wayland |
| mate | MATE | X11 |
| sway | minimal tiling sway | Wayland |
| minimal | none (TTY) | — |

Because the system is declarative, the desktop isn't locked at install. Change `[desktop] environment` in `mycel.toml`, run `mycel switch`, log back in — the session launcher dispatches to whichever DE you named.

## mycel CLI

| Command | Description |
|---|---|
| `mycel switch` | Apply `mycel.toml` — packages, users, services, hostname/locale/timezone |
| `mycel get <pkgs>` | Install packages now and save them to `mycel.toml` |
| `mycel rollback` | Roll back to the previous generation — live, no reboot |
| `mycel rollback <id>` | Roll back to a specific generation |
| `mycel theme [name]` | List / apply a colour theme to the desktop |
| `mycel update` / `mycel check` | Pull overlay cache / show available updates |
| `mycel doctor` | Health check — services, config, package DB, disk |
| `mycel boot <id>` | Set the boot generation for next restart |
| `mycel edit` | Open `mycel.toml` in `$EDITOR` |
| `mycel network` / `mycel active` | List generations / show current system state |
| `mycel diff <a> <b>` | Compare packages between two generations |
| `mycel purge` | Garbage-collect old generations |
| `mycel isolate <id>` / `mycel release <id>` | Pin / unpin a generation |
| `mycel lock <pkg>` / `mycel unlock <pkg>` | Pin / unpin a package across rollbacks |
| `mycel spore <pkgs>` | Ephemeral shell with extra packages — vanishes on exit |
| `mycel spread --export <path>` | Export config for a fresh install |
| `mycel guide` | Built-in guide for new users |

## The .myc package format

Packages are plain TOML. Anyone can write one:

```toml
[package]
name        = "btop"
version     = "1.4.7"
description = "Resource monitor"
license     = "Apache-2.0"
maintainer  = "yourname"
arch        = "x86_64"

[source]
type     = "github-release"
repo     = "aristocratos/btop"
tag      = "v1.4.7"
asset    = "btop-x86_64-unknown-linux-gnu.tar.gz"
checksum = "sha256:..."

[install]
binaries = [{ from = "btop/bin/btop", to = "btop" }]
```

`mycel-pkg` handles install/remove/search/list/info/verify/build/submit, supports source builds (make/cmake/meson/cargo/go) and AppImages, and verifies checksums. The community overlay (`mycel-linux/community`) ships 70+ recipes.

## How services work

Services are defined in `mycel.toml` and managed at runtime by **s6-rc** from a compiled dependency graph, so they start in the right order:

```
udevd → udev-trigger → dbus → elogind → seatd → pipewire → wireplumber → desktop
```

`mycel switch` brings services up or down immediately via `s6-rc change` — no reboot. The init itself is **s6-linux-init** as PID 1, which means `reboot`/`poweroff` work cleanly.

```toml
[services]
enable = ["pipewire", "wireplumber", "NetworkManager", "bluetooth", "cronie"]
```

## Building

```sh
# 1. Build the Rust tools
cd mycel     && cargo build --release && cd ..
cd mycel-pkg && cargo build --release && cd ..
cd fessus/fessus-init && cargo build --release && cd ../..

# 2. Build an ISO (downloads upstream packages as a build-time binary source;
#    the skarnet s6 suite is compiled from source. No pacman in the result.)
cd mycel-iso
sudo bash build.sh                    # KDE Plasma (default)
sudo bash build.sh --profile gnome    # GNOME edition
sudo bash build.sh --profile minimal  # headless, no DE
```

Build host needs: a Rust toolchain, `gcc`/`make`, `curl`, `squashfs-tools`, `libisoburn` (xorriso), `dracut`, `limine`. Each ISO boots to the live desktop with the Calamares installer; installation copies the squashfs straight to disk and is fully offline.

Test in QEMU (UEFI + GPU acceleration for the Wayland compositor):

```sh
qemu-system-x86_64 -enable-kvm -m 4G -smp 2 \
  -cdrom mycel-iso/build/MycelOS-1.0-plasma-x86_64.iso \
  -bios /usr/share/edk2/x64/OVMF.4m.fd \
  -device virtio-vga-gl -display gtk,gl=on
```

## Repo structure

```
mycel-os/
  mycel/              # CLI system manager (Rust)
  mycel-pkg/          # Package manager (Rust)
  mycel-core/
    s6-rc/            # s6-rc service source definitions
    s6-linux-init/    # PID 1 stage scripts (rc.init / rc.shutdown)
    assets/           # logos, wallpaper
  fessus/             # minimal sway config generator (Rust)
  mycel-installer/    # Calamares offline installer config + custom modules
  mycel-iso/          # ISO build system
    bootstrap.sh      # assembles the rootfs from scratch
    build.sh          # squashfs + initramfs + ISO
    profiles/         # one file per desktop edition
  community/          # community overlay index + .myc recipes
```

## Status

**MycelOS boots to a working KDE Plasma 6 desktop** — from a from-scratch rootfs, on s6-linux-init as PID 1, with no systemd.

Working today:
- Full boot chain: Limine → dracut live squashfs → s6-linux-init → s6-rc service graph → elogind/seatd/dbus/pipewire → PAM-registered logind session → Plasma
- `mycel` / `mycel-pkg` CLI tools build and run
- Dependency-resolving build system (full `%DEPENDS%`/`%PROVIDES%` closure)
- skarnet s6 suite built from source; elogind for logind on a no-systemd system
- Per-edition ISOs (Plasma default; GNOME, XFCE, Cinnamon, Budgie, MATE, sway, minimal)
- Generation snapshots + live `mycel rollback`; `mycel doctor` health checks
- 70+ community package recipes

In progress: Calamares install end-to-end verification, per-edition branding, wiring the MycelOS wallpaper/theme into Plasma, and declarative kernel selection.

## License

GPL-3.0 — see [LICENSE](LICENSE)
