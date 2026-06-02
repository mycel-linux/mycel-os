<div align="center">
  <img src="mycel-core/assets/logo_black.png" width="120" />
  <h1>MycelOS</h1>
  <p>Declarative. Reproducible. Truly independent.</p>

  ![License](https://img.shields.io/badge/license-GPL--3.0-blue)
  ![Status](https://img.shields.io/badge/status-early%20development-orange)
  ![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey)
</div>

---

MycelOS is an independent Linux distribution built from scratch. Not a fork. Not based on anything. It has its own package format, its own package manager, and its own desktop environment — all driven by a single declarative file: `mycel.toml`.

Your entire system — packages, services, users, desktop — is declared in one place and applied atomically. Every change creates a new generation you can roll back to. The system is immutable by default.

## What makes it different

Most distros are forks. They live under the decisions of their upstream. MycelOS owns its entire stack:

- **`.myc` packages** — a simple TOML-based package format anyone can write
- **`mycel-pkg`** — a package manager that installs `.myc` packages with checksum verification, desktop integration, and clean removal
- **`mycel`** — the system manager that applies your `mycel.toml` declaratively
- **FessusDE** — a lightweight Wayland desktop built for low-to-mid range hardware
- **runit** — fast, simple init system, no systemd
- **Community overlays** — add any GitHub repo as a package source, no server needed

## mycel.toml

One file. Your whole system.

```toml
[system]
hostname = "mycelbox"
timezone = "America/New_York"
locale   = "en_US.UTF-8"
kernel   = "performance"
immutable = true

[packages]
install = ["firefox", "kitty", "git", "btop"]
lock    = ["firefox"]

[overlays]
sources = [
  "github:mycel-linux/mycel-os",
  "github:yourname/your-packages",
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

Run `mycel switch` to apply it. That's it.

## The .myc package format

Packages are defined in plain TOML. Writing one takes five minutes:

```toml
[package]
name        = "btop"
version     = "1.4.7"
description = "Resource monitor showing CPU, memory, disk and network usage"
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
binaries = [
  { from = "btop/bin/btop", to = "btop" }
]

[desktop]
name       = "btop++"
exec       = "btop"
icon       = "btop"
categories = ["System", "Monitor"]
```

Install it: `mycel-pkg install btop.myc`

## mycel CLI

| Command | Description |
|---|---|
| `mycel switch` | Apply `mycel.toml` to the running system |
| `mycel boot <id>` | Set boot generation for next restart |
| `mycel edit` | Open `mycel.toml` in `$EDITOR` |
| `mycel edit fessus` | Open `fessus.toml` — auto-applies on save |
| `mycel network` | List all system generations |
| `mycel active` | Show current system state |
| `mycel diff <a> <b>` | Compare two generations |
| `mycel purge` | Garbage collect old generations |
| `mycel isolate <id>` | Pin a generation so purge skips it |
| `mycel lock <pkg>` | Pin a package so it survives rollbacks |
| `mycel spore <pkgs>` | Ephemeral shell — vanishes on exit |
| `mycel spread --export <path>` | Export config for a fresh install |
| `mycel guide` | Built-in guide for new users |

## mycel-pkg CLI

| Command | Description |
|---|---|
| `mycel-pkg install <recipe.myc>` | Install a package from a recipe |
| `mycel-pkg remove <name>` | Remove an installed package |
| `mycel-pkg verify <recipe.myc>` | Validate a recipe without installing |
| `mycel-pkg info <recipe.myc>` | Show package metadata |
| `mycel-pkg list` | List all installed packages |
| `mycel-pkg search <query>` | Search the community index |
| `mycel-pkg submit <recipe.myc>` | Prepare a recipe for community submission |

## FessusDE

FessusDE (Latin: *tired*) is a Wayland-native desktop environment designed for low-to-mid range hardware. It composes sway, eww, waybar, and dunst into a cohesive experience with a single configuration file.

Its signature feature is the **radial corner menu** — a hot-corner activated launcher that fans your pinned apps in a quarter-circle arc from the corner of your screen.

Configuration lives in `~/.config/fessus.toml`. Open it with `mycel edit fessus` — changes apply instantly when you save.

```toml
[fessus]
accent_color = "#3F549E"
theme        = "dark"
font         = "Inter"

[radial]
corner = "bottom-left"
pinned = ["firefox", "thunar", "kitty", "mpv"]

[bar]
position     = "top"
show_battery = true
show_network = true

[keybindings]
mod      = "Super"
terminal = "kitty"
```

Supported desktop environments: `fessus`, `hyprland`, `niri`, `plasma`, `gnome`, `xfce`, `cinnamon`, `none`

## Community packages

Anyone can package software for MycelOS. Write a `.myc` recipe, host it on GitHub, and add it to the community index. No server required — the index is just a file in this repo.

To submit a package, read [community/CONTRIBUTING.md](community/CONTRIBUTING.md).

## Repo structure

```
mycel-os/
  mycel/              # CLI system manager (Rust)
  mycel-pkg/          # Package manager (Rust)
  mycel-core/         # runit services, base configs, assets
  fessus/             # FessusDE + fessus-init config generator
  mycel-installer/    # Calamares installer configuration
  mycel-iso/          # ISO build scripts
  community/          # Community overlay index + recipes
```

## Status

MycelOS is in early development. The CLI tools build and run. The package format is defined. The desktop environment is designed. A bootable ISO does not exist yet.

If you want to follow along or contribute, watch this repo.

## License

GPL-3.0 — see [LICENSE](LICENSE)
