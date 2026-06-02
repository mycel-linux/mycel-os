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
| Init | **s6** | Modern, actively maintained, proper supervision |
| Packages | **mycel-pkg + .myc** | Own format, GitHub-native, no foreign tooling |
| Config | **mycel.toml** | One file declares your entire system |
| Desktop | **FessusDE** | Lightweight Wayland DE for real hardware |
| Installer | **Calamares** | Friendly GUI install, no command line required |
| Bootloader | **Limine** | Modern, fast, BIOS + UEFI |

## mycel.toml

One file. Your whole system.

```toml
[system]
hostname = "mycelbox"
timezone = "America/New_York"
kernel   = "performance"
channel  = "stable"

[packages]
install = ["firefox", "kitty", "git", "btop"]
lock    = ["firefox"]

[overlays]
sources = [
  "github:mycel-linux/mycel-os",
]

[desktop]
environment = "fessus"

[services]
enable = ["pipewire", "NetworkManager", "bluetooth"]

[[users]]
name   = "alice"
shell  = "bash"
groups = ["wheel", "audio", "video", "input"]
password_hash = ""
```

Run `mycel switch` to apply. Every change creates a new generation you can roll back to.

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
asset    = "btop-x86_64-unknown-linux-musl.tar.gz"
checksum = "sha256:5099054dd6a101bd12eb6ff3702a9a6a3f57aaa27923a0da478ae5b517faf335"

[install]
binaries = [{ from = "btop/bin/btop", to = "btop" }]
```

Install: `mycel-pkg install btop.myc`

## mycel CLI

| Command | Description |
|---|---|
| `mycel switch` | Apply `mycel.toml` — install packages, reload s6 services |
| `mycel update` | Pull latest overlay cache |
| `mycel check` | Show available updates without applying |
| `mycel boot <id>` | Set boot generation for next restart |
| `mycel edit` | Open `mycel.toml` in `$EDITOR` |
| `mycel edit fessus` | Open `fessus.toml` — auto-applies on save |
| `mycel network` | List all system generations |
| `mycel active` | Show current system state |
| `mycel diff <a> <b>` | Compare two generations |
| `mycel purge` | Garbage collect old generations |
| `mycel isolate <id>` | Pin a generation so purge skips it |
| `mycel lock <pkg>` | Pin a package across rollbacks |
| `mycel spore <pkgs>` | Ephemeral shell — vanishes on exit |
| `mycel spread --export <path>` | Export config for fresh install |
| `mycel guide` | Built-in guide for new users |

## FessusDE

FessusDE (Latin: *tired*) is a Wayland-native desktop for low-to-mid range hardware. It composes sway, eww, waybar, and dunst into a cohesive experience configured entirely from `~/.config/fessus.toml`.

Its signature feature is the **radial corner menu** — a hot-corner launcher that fans your pinned apps in a quarter-circle arc.

```toml
[fessus]
accent_color = "#3F549E"
theme        = "dark"

[radial]
corner = "bottom-left"
pinned = ["firefox", "thunar", "kitty", "mpv"]

[keybindings]
mod      = "Super"
terminal = "kitty"
```

`mycel edit fessus` — changes apply instantly on save.

## Channels

```toml
[system]
channel = "stable"    # stable or unstable
```

- **stable** — tested releases, recommended for most users
- **unstable** — tracks `main`, bleeding edge, for adventurous users

`mycel update` pulls the latest packages for your channel. `mycel check` shows what would change without applying it.

## Community packages

Anyone can package software for MycelOS and list it in the community index. No server required — the index is a file in this repo. See [community/CONTRIBUTING.md](community/CONTRIBUTING.md).

```toml
[overlays]
sources = [
  "github:mycel-linux/mycel-os",
  "github:yourname/your-packages",
]
```

## Repo structure

```
mycel-os/
  mycel/              # CLI system manager (Rust)
  mycel-pkg/          # Package manager (Rust)
  mycel-core/         # s6 services, base configs, assets
  fessus/             # FessusDE + fessus-init config generator
  mycel-installer/    # Calamares installer configuration
  mycel-iso/          # ISO build scripts
  community/          # Community overlay index + recipes
```

## Status

MycelOS is in active early development. The CLI tools build and run. The package manager installs and removes packages. The desktop environment generates configs. A bootable ISO is in progress.

If you want to follow along or contribute, watch this repo.

## License

GPL-3.0 — see [LICENSE](LICENSE)
