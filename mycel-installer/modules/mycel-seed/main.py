import libcalamares
import os

MYCEL_TOML_SUFFIX = "etc/mycel.toml"

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
groups = ["wheel", "audio", "video", "input", "seat", "storage"]
password_hash = "{password_hash}"
"""


def run():
    gs = libcalamares.globalStorage

    root = gs.value("rootMountPoint") or "/mnt"

    hostname = gs.value("hostname") or "mycelbox"

    region = gs.value("locationRegion") or "UTC"
    zone   = gs.value("locationZone")   or ""
    timezone = (region + "/" + zone).strip("/") if zone else region

    locale_conf = gs.value("localeConf") or {}
    locale      = locale_conf.get("LANG", "en_US.UTF-8")

    browser = gs.value("selectedBrowser") or "firefox"
    shell   = gs.value("selectedShell")   or "bash"

    # Calamares stores created users as a list of dicts
    users = gs.value("users") or []
    if users:
        user          = users[0]
        username      = user.get("username", "user")
        password_hash = user.get("cryptedPassword", "")
    else:
        username      = "user"
        password_hash = ""

    content = TEMPLATE.format(
        hostname=hostname,
        timezone=timezone,
        locale=locale,
        browser=browser,
        shell=shell,
        username=username,
        password_hash=password_hash,
    )

    dest = os.path.join(root, MYCEL_TOML_SUFFIX)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    with open(dest, "w") as f:
        f.write(content)

    return None
