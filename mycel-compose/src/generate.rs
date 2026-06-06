use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::schema::Service;

/// Generate a complete s6-rc source tree from a set of service declarations.
pub fn generate(services: &[Service], out: &Path) -> Result<()> {
    if out.exists() {
        fs::remove_dir_all(out).ok();
    }
    fs::create_dir_all(out)?;

    // `ready` / `log` constraints, checked before emitting anything.
    let declared: std::collections::HashSet<&str> =
        services.iter().map(|s| s.name.as_str()).collect();
    for svc in services {
        if svc.ready.is_some() {
            if svc.kind != "longrun" {
                bail!(
                    "service '{}': `ready` only applies to longruns (kind = '{}')",
                    svc.name, svc.kind
                );
            }
            if svc.run.is_some() {
                bail!(
                    "service '{}': `ready` can't combine with a raw `run` — \
                     put s6-notifyoncheck in your run script yourself",
                    svc.name
                );
            }
        }
        if svc.log {
            if svc.kind != "longrun" {
                bail!(
                    "service '{}': `log` only applies to longruns (kind = '{}')",
                    svc.name, svc.kind
                );
            }
            if declared.contains(format!("{}-log", svc.name).as_str()) {
                bail!(
                    "service '{}': `log` generates '{}-log', but a service by that \
                     name already exists",
                    svc.name, svc.name
                );
            }
        }
    }

    // Collect bundle membership declared across services:
    //   bundle name -> set of member service names. A logging service's generated
    //   `<name>-log` consumer joins the same bundles as its producer.
    let mut bundle_members: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for svc in services {
        for b in &svc.bundles {
            bundle_members.entry(b.clone()).or_default().push(svc.name.clone());
            if svc.log {
                bundle_members
                    .entry(b.clone())
                    .or_default()
                    .push(format!("{}-log", svc.name));
            }
        }
    }

    for svc in services {
        let dir = out.join(&svc.name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("type"), format!("{}\n", svc.kind))?;

        match svc.kind.as_str() {
            "longrun" => write_longrun(svc, &dir)?,
            "oneshot" => write_oneshot(svc, &dir)?,
            "bundle"  => {} // contents handled below
            other     => bail!("service '{}': unknown kind '{}'", svc.name, other),
        }

        // dependencies (one per line), for longrun/oneshot
        if !svc.needs.is_empty() && !svc.is_bundle() {
            let mut deps = svc.needs.clone();
            deps.sort();
            deps.dedup();
            fs::write(dir.join("dependencies"), deps.join("\n") + "\n")?;
        }

        // Logger pipeline (F-02): this longrun becomes a producer feeding a
        // generated `<name>-log` consumer that runs s6-log. write_longrun already
        // merged stderr into stdout (2>&1) so both are captured.
        if svc.log && svc.is_longrun() {
            fs::write(dir.join("producer-for"), format!("{}-log\n", svc.name))?;
            write_logger(&svc.name, &out.join(format!("{}-log", svc.name)))?;
        }
    }

    // Write bundle contents = explicit `contents` + collected members.
    for svc in services.iter().filter(|s| s.is_bundle()) {
        let dir = out.join(&svc.name);
        let mut members = svc.contents.clone();
        if let Some(extra) = bundle_members.get(&svc.name) {
            members.extend(extra.iter().cloned());
        }
        members.sort();
        members.dedup();
        if members.is_empty() {
            bail!("bundle '{}' has no contents", svc.name);
        }
        fs::write(dir.join("contents"), members.join("\n") + "\n")?;
    }

    // Sanity: every dependency / bundle member must resolve to a real service.
    validate(services)?;

    Ok(())
}

fn write_longrun(svc: &Service, dir: &Path) -> Result<()> {
    let body = if let Some(raw) = &svc.run {
        // Verbatim escape hatch.
        let mut s = raw.clone();
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    } else {
        let exec = svc.exec.as_deref().with_context(|| {
            format!("longrun '{}' needs an `exec` (or a raw `run`)", svc.name)
        })?;

        // The daemon command, with optional privilege drop.
        let daemon = match svc.user.as_deref() {
            Some(u) if !u.is_empty() => format!("s6-setuidgid {} {}", u, exec),
            _ => exec.to_string(),
        };

        let mut s = String::from("#!/bin/sh\n");
        for line in &svc.setup {
            s.push_str(line);
            s.push('\n');
        }
        // When logging, merge stderr into stdout so both reach the logger pipe.
        let redir = if svc.log { " 2>&1" } else { "" };
        if svc.ready.is_some() {
            // Readiness-gated: s6-notifyoncheck runs ./data/check until it
            // succeeds, then signals readiness on fd 3 (see notification-fd
            // below), so s6-rc holds dependents until the service is serving.
            s.push_str(&format!(
                "exec s6-notifyoncheck -t 1500 -w 250 -T 30000 {}{}\n",
                daemon, redir
            ));
        } else {
            s.push_str(&format!("exec {}{}\n", daemon, redir));
        }
        s
    };

    let run = dir.join("run");
    fs::write(&run, body)?;
    fs::set_permissions(&run, fs::Permissions::from_mode(0o755))?;

    // Readiness wiring: mark the service notification-aware and emit the probe
    // s6-notifyoncheck polls. (`ready` + raw `run` is rejected in generate().)
    if let Some(spec) = &svc.ready {
        fs::write(dir.join("notification-fd"), "3\n")?;
        let data = dir.join("data");
        fs::create_dir_all(&data)?;
        let check = data.join("check");
        fs::write(&check, ready_check_script(&svc.name, spec)?)?;
        fs::set_permissions(&check, fs::Permissions::from_mode(0o755))?;
    }

    if let Some(fin) = &svc.finish {
        let path = dir.join("finish");
        let mut s = String::from("#!/bin/sh\n");
        s.push_str(fin);
        if !s.ends_with('\n') {
            s.push('\n');
        }
        fs::write(&path, s)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

/// Build the `data/check` script body for a `ready = "<kind>:<arg>"` spec. The
/// script must exit 0 once the service is actually serving; s6-notifyoncheck
/// polls it and signals readiness when it first succeeds.
fn ready_check_script(name: &str, spec: &str) -> Result<String> {
    let (kind, arg) = spec.split_once(':').with_context(|| {
        format!(
            "service '{}': `ready` must be '<kind>:<arg>' (dbus:/path:/exec:), got '{}'",
            name, spec
        )
    })?;
    let check = match kind {
        // Wait until the bus name is owned on the system bus.
        "dbus" => format!(
            "dbus-send --system --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.NameHasOwner string:{} 2>/dev/null | grep -q true",
            arg
        ),
        // Wait until a path (socket, pidfile, …) exists.
        "path" => format!("test -e {}", arg),
        // Arbitrary shell readiness command, verbatim.
        "exec" => arg.to_string(),
        other => bail!(
            "service '{}': unknown `ready` kind '{}' (use dbus:/path:/exec:)",
            name, other
        ),
    };
    Ok(format!("#!/bin/sh\n{}\n", check))
}

fn write_oneshot(svc: &Service, dir: &Path) -> Result<()> {
    let up = svc.up.as_ref().with_context(|| {
        format!("oneshot '{}' needs an `up` action", svc.name)
    })?;
    // Cmd::render emits valid execline: a single command verbatim, or a list as
    // one `/bin/sh -c` script — so multi-step oneshots need no hand-rolled wrap.
    fs::write(dir.join("up"), up.render())?;
    if let Some(down) = &svc.down {
        fs::write(dir.join("down"), down.render())?;
    }
    Ok(())
}

/// Generate the `<name>-log` consumer service that logs a producer's piped
/// stdout/stderr via s6-log into /var/log/<name>.
fn write_logger(producer: &str, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("type"), "longrun\n")?;
    fs::write(dir.join("consumer-for"), format!("{}\n", producer))?;
    let run = format!(
        "#!/bin/sh\nmkdir -p /var/log/{p}\nexec s6-log -b n10 s1000000 T /var/log/{p}\n",
        p = producer
    );
    let rp = dir.join("run");
    fs::write(&rp, run)?;
    fs::set_permissions(&rp, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

/// Every name referenced in `needs` or a bundle's `contents` must be a real
/// declared service. Catches typos before s6-rc-compile ever runs.
fn validate(services: &[Service]) -> Result<()> {
    let names: std::collections::HashSet<&str> =
        services.iter().map(|s| s.name.as_str()).collect();

    for svc in services {
        for dep in &svc.needs {
            if !names.contains(dep.as_str()) {
                bail!("service '{}' depends on unknown service '{}'", svc.name, dep);
            }
        }
        for m in &svc.contents {
            if !names.contains(m.as_str()) {
                bail!("bundle '{}' references unknown service '{}'", svc.name, m);
            }
        }
        for b in &svc.bundles {
            if !names.contains(b.as_str()) {
                bail!("service '{}' wants to join unknown bundle '{}'", svc.name, b);
            }
        }
    }
    Ok(())
}
