import libcalamares
import os
import shutil
import subprocess

LIVE_PATHS = [
    "home/live",
    "root/.bash_history",
    "root/.lesshst",
]


def _write_locale_gen(root, locale_conf_path):
    """Read LANG from /etc/locale.conf and write it to /etc/locale.gen."""
    lang = None
    try:
        with open(locale_conf_path) as f:
            for line in f:
                line = line.strip()
                if line.startswith("LANG="):
                    lang = line.split("=", 1)[1].strip().strip('"')
                    break
    except OSError:
        return

    if not lang:
        return

    # Strip the encoding suffix (e.g. "en_US.UTF-8" → "en_US") to build the
    # locale.gen entry, then re-add UTF-8 in the standard format.
    locale_gen_entry = f"{lang} UTF-8\n"
    locale_gen_path  = os.path.join(root, "etc", "locale.gen")

    existing = ""
    try:
        with open(locale_gen_path) as f:
            existing = f.read()
    except OSError:
        pass

    if locale_gen_entry not in existing:
        with open(locale_gen_path, "a") as f:
            f.write(locale_gen_entry)


def run():
    gs   = libcalamares.globalStorage
    root = gs.value("rootMountPoint") or "/mnt"

    # ── locale.gen ────────────────────────────────────────────────────────────
    # Calamares' locale module writes /etc/locale.conf but not /etc/locale.gen.
    # We write it here so that locale-gen on first boot actually generates the
    # right locale instead of silently doing nothing.
    _write_locale_gen(root, os.path.join(root, "etc", "locale.conf"))

    # ── fessus.toml for installed user ────────────────────────────────────────
    users    = gs.value("users") or []
    user     = users[0] if users else {}
    username = user.get("username", "")
    uid      = user.get("uid", 1000)
    gid      = user.get("gid", 1000)

    if username:
        user_home   = os.path.join(root, "home", username)
        user_config = os.path.join(user_home, ".config")
        live_fessus = os.path.join(root, "home", "live", ".config", "fessus.toml")
        dest_fessus = os.path.join(user_config, "fessus.toml")
        os.makedirs(user_config, exist_ok=True)
        if os.path.exists(live_fessus) and not os.path.exists(dest_fessus):
            shutil.copy2(live_fessus, dest_fessus)
        try:
            os.chown(user_home,   uid, gid)
            os.chown(user_config, uid, gid)
            if os.path.exists(dest_fessus):
                os.chown(dest_fessus, uid, gid)
        except OSError:
            pass

    # ── remove live-only paths ────────────────────────────────────────────────
    for rel in LIVE_PATHS:
        path = os.path.join(root, rel)
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.rmtree(path, ignore_errors=True)
        elif os.path.exists(path) or os.path.islink(path):
            try:
                os.remove(path)
            except OSError:
                pass

    subprocess.run(["chroot", root, "userdel", "-r", "live"], check=False)

    # ── machine-id ────────────────────────────────────────────────────────────
    mid = os.path.join(root, "etc", "machine-id")
    try:
        with open(mid, "w") as f:
            f.write("uninitialized\n")
    except OSError:
        pass

    # ── firstboot sentinel ────────────────────────────────────────────────────
    sentinel = os.path.join(root, "var", "lib", "mycel", "firstboot.done")
    try:
        os.remove(sentinel)
    except OSError:
        pass

    return None
