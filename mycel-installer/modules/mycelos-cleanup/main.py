import libcalamares
import os
import shutil
import subprocess

LIVE_PATHS = [
    "home/live",
    "root/.bash_history",
    "root/.lesshst",
]


def _warn(msg):
    try:
        libcalamares.utils.warning("mycelos-cleanup: " + msg)
    except Exception:
        pass


def _write_locale_gen(root):
    """Mirror LANG from /etc/locale.conf into /etc/locale.gen."""
    locale_conf = os.path.join(root, "etc", "locale.conf")
    lang = None
    try:
        with open(locale_conf) as f:
            for line in f:
                line = line.strip()
                if line.startswith("LANG="):
                    lang = line.split("=", 1)[1].strip().strip('"')
                    break
    except OSError:
        return
    if not lang:
        return

    entry = "{} UTF-8\n".format(lang)
    path  = os.path.join(root, "etc", "locale.gen")
    existing = ""
    try:
        with open(path) as f:
            existing = f.read()
    except OSError:
        pass
    if entry not in existing:
        with open(path, "a") as f:
            f.write(entry)


def _seed_user_config(root):
    """Copy the live fessus.toml into the new user's home, fix ownership."""
    users    = libcalamares.globalStorage.value("users") or []
    user     = users[0] if users else {}
    username = user.get("username", "") if isinstance(user, dict) else ""
    if not username:
        return

    # uid/gid may come back as str, None, or be absent — coerce to int safely.
    try:
        uid = int(user.get("uid", 1000))
    except (TypeError, ValueError):
        uid = 1000
    try:
        gid = int(user.get("gid", 1000))
    except (TypeError, ValueError):
        gid = 1000

    user_home   = os.path.join(root, "home", username)
    user_config = os.path.join(user_home, ".config")
    live_fessus = os.path.join(root, "home", "live", ".config", "fessus.toml")
    dest_fessus = os.path.join(user_config, "fessus.toml")

    os.makedirs(user_config, exist_ok=True)
    if os.path.exists(live_fessus) and not os.path.exists(dest_fessus):
        shutil.copy2(live_fessus, dest_fessus)

    for p in (user_home, user_config, dest_fessus):
        if os.path.exists(p):
            os.chown(p, uid, gid)


def _remove_live_paths(root):
    for rel in LIVE_PATHS:
        path = os.path.join(root, rel)
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.rmtree(path, ignore_errors=True)
        elif os.path.exists(path) or os.path.islink(path):
            try:
                os.remove(path)
            except OSError:
                pass


def run():
    """Tidy the target after unpack. Every step is best-effort: a failure here
    must never abort a successful install, so we catch everything and warn."""
    root = libcalamares.globalStorage.value("rootMountPoint") or "/mnt"

    for label, fn in (
        ("locale.gen",     lambda: _write_locale_gen(root)),
        ("user config",    lambda: _seed_user_config(root)),
        ("remove live",    lambda: _remove_live_paths(root)),
        ("userdel live",   lambda: subprocess.run(
            ["chroot", root, "userdel", "-r", "live"], check=False)),
        ("machine-id",     lambda: open(os.path.join(root, "etc", "machine-id"), "w")
                                     .write("uninitialized\n")),
        ("firstboot",      lambda: os.remove(
            os.path.join(root, "var", "lib", "mycel", "firstboot.done"))),
    ):
        try:
            fn()
        except Exception as e:
            _warn("{} step failed (non-fatal): {}".format(label, e))

    return None
