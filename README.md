<div align="center">
  <img src="mycel-core/assets/logo_black.png" width="120" />
  <h1>MycelOS</h1>
  <p>Declarative. Reproducible. Community powered.</p>

  ![License](https://img.shields.io/badge/license-GPL--3.0-blue)
  ![Status](https://img.shields.io/badge/status-early%20development-orange)
  ![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey)
</div>

---

MycelOS is an independent Linux distribution built from scratch, powered by [nixpkgs](https://github.com/NixOS/nixpkgs) and driven by a single declarative file: `mycel.toml`. Your entire system — packages, services, users, desktop — is declared in one place and applied atomically.

It ships with **FessusDE**, a lightweight Wayland desktop environment built for low-to-mid range hardware, featuring a signature radial corner menu and a one-file configuration system.

## Features

- **Declarative** — your whole system lives in `/etc/mycel.toml`
- **Reproducible** — every change creates a new generation you can boot into or roll back from
- **Immutable root** — the system is read-only by default
- **nixpkgs powered** — access to one of the largest package collections in Linux
- **runit init** — fast, simple, no systemd
- **FessusDE** — a Wayland-native desktop that runs well on older hardware
- **Community overlays** — add any GitHub repo as a package source, no server needed

## mycel.toml

```toml
[system]
hostname = "mycelbox"
timezone = "America/New_York"
kernel = "performance"
immutable = true

[packages]
install = ["firefox", "kitty", "git"]
lock = ["firefox"]

[desktop]
environment = "fessus"

[services]
enable = ["pipewire", "NetworkManager", "bluetooth"]

[[users]]
name = "alice"
shell = "bash"
groups = ["wheel", "audio", "video"]
password_hash = ""
```

Apply it with `mycel switch`. That's it.

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
| `mycel purge` | Garbage collect `/nix/store` |
| `mycel isolate <id>` | Pin a generation from GC |
| `mycel lock <pkg>` | Pin a package across rollbacks |
| `mycel spore <pkgs>` | Ephemeral shell, vanishes on exit |
| `mycel spread --export <path>` | Export config for a fresh install |
| `mycel guide` | Built-in guide for new users |

## FessusDE

FessusDE (Latin: *tired*) is a lightweight Wayland desktop environment that composes battle-tested tools — sway, eww, waybar, dunst — into a cohesive, easy to configure experience.

Its signature feature is the **radial corner menu**: a hot-corner activated launcher that fans your pinned apps in a quarter-circle arc.

Configuration lives in `~/.config/fessus.toml`. Run `mycel edit fessus` to open it — changes apply to the desktop instantly on save.

```toml
[fessus]
accent_color = "#3F549E"
theme = "dark"
font = "Inter"

[radial]
corner = "bottom-left"
pinned = ["firefox", "thunar", "kitty", "mpv"]

[bar]
position = "top"
show_battery = true
show_network = true
```

## Community packages

Add any GitHub repo as a package overlay in `mycel.toml`:

```toml
[overlays]
sources = [
  "github:mycel-linux/community",
  "github:yourname/your-packages",
]
```

To list your overlay in the official community index, open a PR to [mycel-linux/mycel-os](https://github.com/mycel-linux/mycel-os) adding your entry to `community/index.toml`.

## Repo structure

```
mycel-os/
  mycel/            # CLI system manager (Rust)
  mycel-core/       # runit services, base configs, assets
  fessus/           # FessusDE + fessus-init config generator
  mycel-installer/  # Calamares installer configuration
  mycel-iso/        # ISO build scripts
  community/        # Community overlay index
```

## Status

MycelOS is in early development. Nothing is installable yet. If you want to follow progress or contribute, watch this repo.

## License

GPL-3.0 — see [LICENSE](LICENSE)
