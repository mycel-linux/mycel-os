use serde::Deserialize;

/// A oneshot up/down action: either a single command line (kept verbatim as
/// execline) or a list of commands run in sequence (rendered as one `/bin/sh -c`
/// script). The list form removes the hand-rolled `/bin/sh -c "a; b; c"` dance.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Cmd {
    One(String),
    Many(Vec<String>),
}

impl Cmd {
    /// Contents of a oneshot `up`/`down` file (valid execline either way).
    pub fn render(&self) -> String {
        match self {
            Cmd::One(s) => format!("{}\n", s),
            Cmd::Many(lines) => {
                // execline execs `/bin/sh -c "<body>"`; the body is a normal
                // shell script, one command per line. Escape \ and " for the
                // execline double-quoted word.
                let mut body = String::from("/bin/sh -c \"\n");
                for l in lines {
                    body.push_str(&l.replace('\\', "\\\\").replace('"', "\\\""));
                    body.push('\n');
                }
                body.push_str("\"\n");
                body
            }
        }
    }
}

/// One declarative service definition. Lives in a `.toml` file; the composer
/// turns it into a full s6-rc service directory (and, later, dbus/polkit/
/// sysusers glue). The whole point: declare a service ONCE, at a high level,
/// and let the composer emit every fiddly bit of supervision + IPC plumbing.
#[derive(Deserialize, Debug, Clone)]
pub struct Service {
    /// Service name (also the s6-rc service-dir name).
    pub name: String,

    /// "longrun" (a supervised daemon), "oneshot" (run once), or "bundle"
    /// (a named group of other services).
    #[serde(default = "default_type")]
    pub kind: String,

    // ── longrun ────────────────────────────────────────────────────────────
    /// The daemon command line. Generated into an `exec` in the run script.
    pub exec: Option<String>,
    /// Shell lines run before `exec` in the run script (mkdir, chown, etc.).
    #[serde(default)]
    pub setup: Vec<String>,
    /// Run the daemon as this user (via s6-setuidgid). Empty = root.
    pub user: Option<String>,
    /// Optional finish script body (runs when the service stops).
    pub finish: Option<String>,
    /// Escape hatch: a complete run script body. If set, it's used verbatim
    /// and `exec`/`setup`/`user` are ignored. For services too gnarly to model.
    pub run: Option<String>,

    /// Readiness check (longrun only). When set, the daemon is wrapped with
    /// `s6-notifyoncheck`, a `data/check` probe is emitted, and `notification-fd`
    /// is set to 3 — so s6-rc gates dependents on the service being *ready*
    /// (serving), not merely *started*. Removes hand-rolled `*-ready` oneshots.
    /// Forms:
    ///   "dbus:NAME" — wait until bus name NAME is owned on the system bus
    ///   "path:/p"   — wait until path /p exists
    ///   "exec:CMD"  — wait until shell command CMD exits 0
    pub ready: Option<String>,

    /// Attach a logger (longrun only). When true, the composer pipes this
    /// service's stdout+stderr to a generated sibling `<name>-log` consumer that
    /// runs `s6-log` into /var/log/<name> — so output isn't lost under `-B`
    /// (no catch-all logger). Removes hand-rolled logger services.
    #[serde(default)]
    pub log: bool,

    // ── oneshot ────────────────────────────────────────────────────────────
    /// A oneshot's `up` action: a single command, or a list run in sequence.
    pub up: Option<Cmd>,
    /// Optional `down` action (same forms as `up`).
    pub down: Option<Cmd>,

    // ── bundle ─────────────────────────────────────────────────────────────
    /// Explicit members, for `kind = "bundle"`.
    #[serde(default)]
    pub contents: Vec<String>,

    // ── wiring ─────────────────────────────────────────────────────────────
    /// Services this one depends on (s6-rc `dependencies`).
    #[serde(default)]
    pub needs: Vec<String>,
    /// Bundles this service should be a member of. The composer collects these
    /// across all services and folds them into each bundle's contents.
    #[serde(default)]
    pub bundles: Vec<String>,

    // ── phase 2 (parsed now, generated later) ──────────────────────────────
    /// System bus name this service owns → generates a dbus system.d policy.
    #[allow(dead_code)]
    pub dbus_name: Option<String>,
    /// polkit action prefix to allow for the wheel group.
    #[allow(dead_code)]
    pub polkit_allow: Option<String>,
}

fn default_type() -> String {
    "longrun".to_string()
}

impl Service {
    pub fn is_bundle(&self) -> bool {
        self.kind == "bundle"
    }
    pub fn is_oneshot(&self) -> bool {
        self.kind == "oneshot"
    }
    pub fn is_longrun(&self) -> bool {
        self.kind == "longrun"
    }
}
