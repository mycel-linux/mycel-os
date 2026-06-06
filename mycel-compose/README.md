# mycel-compose

**The MycelOS declarative service composer.**

You describe a service *once*, at a high level, in a `.toml` file. mycel-compose
weaves it into all the fiddly supervision + IPC glue that normally has to be
hand-rolled: the s6-rc service directory, dependency edges, bundle membership,
and (roadmap) dbus policy, polkit rules, and sysusers entries.

The whole reason this exists: stitching services together by hand is where
things break — a daemon needs dbus *and* udev up first, needs a runtime dir
owned by the right user, needs a bus policy and a polkit action to be useful.
mycel-compose makes that a matter of editing one declaration instead of editing
five files in three formats and hoping they agree.

It builds *on* proven primitives (s6, s6-rc, dbus, elogind) — it never replaces
them. A bug here is a misconfigured service, not a bricked boot.

## Usage

```sh
# Weave declarations into an s6-rc source tree
mycel-compose --services ./services --out ./s6-rc-source

# Validate declarations without writing anything
mycel-compose --services ./services --check
```

In the ISO build, `bootstrap.sh` calls this to generate
`/etc/s6-rc/source` from `mycel-core/services/*.toml`, then runs
`s6-rc-compile` on the result.

## Declaration schema

One file per service, `<name>.toml`:

```toml
name    = "networkmanager"   # also the s6-rc service-dir name
kind    = "longrun"          # longrun | oneshot | bundle  (default: longrun)

# ── longrun ──────────────────────────────────────────────
exec    = "NetworkManager --no-daemon"   # the daemon command line
setup   = ["mkdir -p /run/foo"]          # shell lines run before exec
user    = "alice"                        # run as this user (s6-setuidgid); empty = root
finish  = "echo bye"                     # optional finish-script body

# Escape hatch for services too stateful for the simple model:
# run   = """#!/bin/sh ... exec ..."""   # verbatim run script; overrides exec/setup/user

# log   = true                 # pipe stdout+stderr to a generated <name>-log
                               # (s6-log -> /var/log/<name>); folded into bundles

# ── oneshot ──────────────────────────────────────────────
# up    = "udevadm settle"               # single command (execline, verbatim)
# up    = ["cmd one", "cmd two", "cmd3"] # OR a list run in sequence (no /bin/sh -c by hand)
# down  = "..."                          # same forms as up

# ── wiring ───────────────────────────────────────────────
needs   = ["dbus", "udevd"]   # s6-rc dependencies
bundles = ["default"]         # bundles this service joins (folded in automatically)

# ── readiness (longrun) ──────────────────────────────────
# Gate dependents on the service actually serving, not just having started.
# ready = "dbus:org.freedesktop.login1"   # wait until the bus name is owned
# ready = "path:/run/foo.sock"            # wait until a path exists
# ready = "exec:mytool --healthcheck"     # wait until a shell command exits 0

# ── bundle (kind = "bundle") ─────────────────────────────
# contents = ["a", "b"]       # explicit members (plus anyone who lists this in `bundles`)

# ── phase 2 (parsed, not yet generated) ──────────────────
# dbus_name    = "org.freedesktop.NetworkManager"
# polkit_allow = "org.freedesktop.NetworkManager"
```

Bundle membership is *collected*: a service lists the bundles it belongs to in
`bundles = [...]`, and the composer folds it into each bundle's contents. You
never edit a bundle's member list by hand.

Before writing anything, the composer validates that every name in `needs`,
`contents`, and `bundles` resolves to a real declared service — typos are caught
ahead of `s6-rc-compile`.

## Roadmap

- **v1 (done):** declarations → s6-rc source tree (run/up/down, type,
  dependencies, finish, bundle contents) + cross-reference validation.
- **v2:** `dbus_name` → a `/usr/share/dbus-1/system.d/` policy granting the
  service's user the right to own that bus name.
- **v2:** `polkit_allow` → a `/etc/polkit-1/rules.d/` rule allowing the wheel
  group the named action prefix.
- **v2:** `user` on a service that needs a system account → a sysusers.d entry.
- **v3 (done):** readiness checks. `ready = "dbus:NAME"` / `"path:/p"` /
  `"exec:CMD"` on a longrun emits `notification-fd` (3) + a `data/check` probe and
  wraps the daemon in `s6-notifyoncheck`, so s6-rc holds dependents until the
  service is *actually serving*, not merely *started*. Removes hand-rolled
  `*-ready` oneshots. (Motivated by Voi6 booting: autologin raced elogind
  acquiring org.freedesktop.login1 → no logind session.)
