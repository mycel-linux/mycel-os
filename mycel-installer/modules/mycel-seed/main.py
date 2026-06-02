import libcalamares
import os

MYCEL_TOML_PATH = "/etc/mycel.toml"

TEMPLATE = """\
[system]
hostname = "{hostname}"
timezone = "{timezone}"
locale = "{locale}"
kernel = "performance"
immutable = true

[boot]
timeout = 3
cmdline = ["quiet", "splash"]

[packages]
install = [
  "{browser}",
  "kitty",
  "thunar",
  "mousepad",
  "mpv",
  "imv",
  "zathura",
  "zathura-pdf-mupdf",
  "btop",
  "fastfetch",
  "xarchiver",
  "blueman",
  "qalculate-gtk",
  "wf-recorder",
  "grim",
  "slurp",
  "cliphist",
  "wl-clipboard",
  "git",
  "curl",
  "wget",
]
lock = []

[overlays]
sources = [
  "github:mycel-linux/community",
]

[desktop]
environment = "fessus"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
  "cronie",
]

[[users]]
name = "{username}"
shell = "{shell}"
groups = ["wheel", "audio", "video", "input", "storage"]
password_hash = "{password_hash}"
"""


def run():
    gs = libcalamares.globalStorage

    hostname      = gs.value("hostname") or "mycelbox"
    timezone      = gs.value("locationRegion", "UTC") + "/" + gs.value("locationZone", "")
    locale        = gs.value("localeConf", {}).get("LANG", "en_US.UTF-8")
    browser       = gs.value("selectedBrowser") or "firefox"
    shell         = gs.value("selectedShell") or "bash"
    username      = gs.value("username") or "user"
    password_hash = gs.value("passwordHash") or ""

    timezone = timezone.strip("/")

    content = TEMPLATE.format(
        hostname=hostname,
        timezone=timezone,
        locale=locale,
        browser=browser,
        shell=shell,
        username=username,
        password_hash=password_hash,
    )

    os.makedirs(os.path.dirname(MYCEL_TOML_PATH), exist_ok=True)
    with open(MYCEL_TOML_PATH, "w") as f:
        f.write(content)

    return None
