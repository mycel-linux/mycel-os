<div align="center">
  <img src="mycel-core/assets/logo_black.png" width="120" />
  <h1>MycelOS</h1>
  <p>Independent. Declarative. s6.</p>

  ![License](https://img.shields.io/badge/license-GPL--3.0-blue)
  ![Status](https://img.shields.io/badge/status-early%20development-orange)
  ![Init](https://img.shields.io/badge/init-s6-purple)
  ![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey)
</div>

---

MycelOS is an independent Linux distribution built from scratch. Not a fork. Not based on anything. It runs on **s6** — a modern, actively maintained process supervision suite — and is driven by a single declarative file: `mycel.toml`.

No systemd. No runit. No OpenRC. Just s6 — fast, clean, and built for the future.

## Why s6?

The anti-systemd space is crowded but stagnant. Void Linux and Artix run runit — a supervision suite last seriously updated in 2004. Alpine and Gentoo run OpenRC — a service manager, not a real supervisor. These are respected choices but they are old choices.

s6 is different. Written by Laurent Bercot and actively maintained, s6 is a proper process supervision suite with clean C code, real dependency handling via s6-rc, and a design that hasn't accumulated 20 years of technical debt. MycelOS is one of the first independent distributions to be built ground-up with s6 as the init system.

If you are done with systemd and done with aging alternatives, MycelOS is built for you.

## The full stack

| Component | Choice | Why |
|---|---|---|
| Init | **s6 + s6-rc** | Modern, actively maintained, proper dependency-ordered supervision |
| Packages | **mycel-pkg + .myc** | Own format, GitHub-native, no foreign tooling |
| Config | **mycel.toml** | One file declares your entire system |
| Desktop | **FessusDE** | Lightweight Wayland compositor built on sway |
| Installer | **Calamares** | Offline GUI install, no command line required |
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
sources = [
  "github:mycel-linux/community",
]

[desktop]
environment = "fessus"

[services]
enable = ["pipewire", "wireplumber", "NetworkManager", "bluetooth"]

[[users]]
name   = "alice"
shell  = "bash"
groups = ["wheel", "audio", "video", "input", "seat"]
password_hash = ""
```

Run `mycel switch` to apply. Every change creates a new generation you can roll back to.

## mycel CLI

| Command | Description |
|---|---|
| `mycel switch` | Apply `mycel.toml` — packages, users, services, system config |
| `mycel get <pkgs>` | Install packages immediately and save them to `mycel.toml` |
| `mycel update` | Pull latest overlay cache |
| `mycel check` | Show available updates without applying |
| `mycel doctor` | Check system health — services, config, DB, disk |
| `mycel boot <id>` | Set boot generation for next restart |
| `mycel edit` | Open `mycel.toml` in `$EDITOR` |
| `mycel edit fessus` | Open `fessus.toml` — desktop changes apply on save |
| `mycel network` | List all system generations |
| `mycel active` | Show current system state |
| `mycel diff <a> <b>` | Compare packages between two generations |
| `mycel purge` | Garbage collect old generations |
| `mycel isolate <id>` | Pin a generation so purge skips it |
| `mycel release <id>` | Unpin a generation |
| `mycel lock <pkg>` | Pin a package across rollbacks |
| `mycel unlock <pkg>` | Remove a package pin |
| `mycel spore <pkgs>` | Ephemeral shell with extra packages — vanishes on exit |
| `mycel spread --export <path>` | Export config for fresh install |
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

### mycel-pkg commands

| Command | Description |
|---|---|
| `mycel-pkg install <recipe>` | Install a package from a .myc file |
| `mycel-pkg remove <name>` | Remove an installed package |
| `mycel-pkg search <query>` | Search recipes in cached overlays |
| `mycel-pkg list` | List all installed packages |
| `mycel-pkg info <recipe>` | Show package metadata |
| `mycel-pkg verify <recipe>` | Validate a .myc recipe |
| `mycel-pkg build <recipe>` | Build a binary .mpkg from a source recipe |
| `mycel-pkg submit <recipe>` | Get submission instructions for the community index |

## FessusDE

FessusDE is a Wayland-native desktop for low-to-mid range hardware. It composes sway, waybar, eww, dunst, and wofi into a cohesive experience configured entirely from `~/.config/fessus.toml`.

Its signature feature is the **radial corner menu** — a hot-corner launcher that fans your pinned apps in a quarter-circle arc, toggled with `Super+r`.

```toml
[fessus]
accent_color = "#3F549E"
theme        = "dark"
font         = "Inter"

[radial]
corner = "bottom-left"
pinned = ["firefox", "thunar", "kitty", "mpv"]

[keybindings]
mod      = "Super"
terminal = "kitty"
```

`mycel edit fessus` — changes apply instantly on save.

## s6 service management

Services are defined in `mycel.toml` and managed at runtime by s6-rc. MycelOS uses a compiled dependency graph so services start in the right order — udevd before dbus, dbus and seatd before pipewire, pipewire before wireplumber, everything before the desktop.

```toml
[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
  "cronie",
]
```

Running `mycel switch` after changing the services list starts or stops services immediately without rebooting.

## Channels

```toml
[system]
channel = "stable"    # stable or unstable
```

- **stable** — tested releases, recommended for most users
- **unstable** — tracks `main`, bleeding edge

`mycel update` pulls the latest packages for your channel. `mycel check` shows what would change without applying it.

## Community packages

Anyone can package software for MycelOS. The community overlay lives at `mycel-linux/community` and already includes 70+ packages. See [community/CONTRIBUTING.md](community/CONTRIBUTING.md).

```toml
[overlays]
sources = [
  "github:mycel-linux/community",
  "github:yourname/your-packages",
]
```

Publish your own recipe:

```sh
mycel-pkg verify myapp.myc    # check it first
mycel-pkg submit myapp.myc    # get submission instructions
```

## Repo structure

```
mycel-os/
  mycel/              # CLI system manager (Rust)
  mycel-pkg/          # Package manager (Rust)
  mycel-core/
    s6-rc/            # s6-rc service source definitions
    s6-linux-init/    # PID 1 init stage scripts
    assets/           # logos, wallpaper
    etc/              # base configs (fastfetch, etc.)
  fessus/             # FessusDE config generator (Rust)
  mycel-installer/    # Calamares offline installer config
  mycel-iso/          # ISO build scripts (bootstrap.sh + build.sh)
  community/          # Community overlay index + recipes
```

## Building

```sh
# Build Rust tools first
cd mycel     && cargo build --release && cd ..
cd mycel-pkg && cargo build --release && cd ..
cd fessus/fessus-init && cargo build --release && cd ../..

# Build the ISO (downloads Arch packages as binary source, no Arch installed)
cd mycel-iso && sudo bash build.sh
```

The ISO boots directly into FessusDE. Click the installer icon on the desktop to install to disk. Installation is fully offline — no network required.

## Status

MycelOS is in active early development. The core systems are functional:

- CLI tools (`mycel`, `mycel-pkg`) build and run
- s6-rc service graph with proper dependency ordering and readiness notification
- s6-linux-init as PID 1 — clean shutdown and reboot
- Calamares offline installer with custom modules
- FessusDE desktop generates from `fessus.toml`
- 70+ community package recipes
- Generation snapshots with btrfs rollback
- `mycel doctor` for system health checks

A bootable ISO is the current focus.

## License

GPL-3.0 — see [LICENSE](LICENSE)
