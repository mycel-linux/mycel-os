use anyhow::{bail, Result};
use std::process::Command;
use crate::package::schema::Build;

pub fn run(build: &Build, src_dir: &str) -> Result<()> {
    match build.system.as_str() {
        "make"   => run_make(build, src_dir),
        "cmake"  => run_cmake(build, src_dir),
        "meson"  => run_meson(build, src_dir),
        "cargo"  => run_cargo(src_dir),
        "go"     => run_go(src_dir),
        "script" => run_script(src_dir),
        other    => bail!("unknown build system: {}", other),
    }
}

fn run_make(build: &Build, dir: &str) -> Result<()> {
    let default = vec![
        "make".to_string(),
        "make install PREFIX=/usr".to_string(),
    ];
    let commands = build.commands.as_deref().unwrap_or(&default);
    run_commands(commands, dir)
}

fn run_cmake(build: &Build, dir: &str) -> Result<()> {
    let default = vec![
        "cmake -B build -DCMAKE_INSTALL_PREFIX=/usr".to_string(),
        "cmake --build build".to_string(),
        "cmake --install build".to_string(),
    ];
    let commands = build.commands.as_deref().unwrap_or(&default);
    run_commands(commands, dir)
}

fn run_meson(build: &Build, dir: &str) -> Result<()> {
    let default = vec![
        "meson setup build --prefix=/usr".to_string(),
        "ninja -C build".to_string(),
        "ninja -C build install".to_string(),
    ];
    let commands = build.commands.as_deref().unwrap_or(&default);
    run_commands(commands, dir)
}

fn run_cargo(dir: &str) -> Result<()> {
    run_commands(&["cargo build --release".to_string()], dir)
}

fn run_go(dir: &str) -> Result<()> {
    run_commands(&["go build -o /usr/bin/$(basename $(pwd))".to_string()], dir)
}

fn run_script(dir: &str) -> Result<()> {
    run_commands(&["bash install.sh".to_string()], dir)
}

fn run_commands(commands: &[String], dir: &str) -> Result<()> {
    for cmd in commands {
        let status = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(dir)
            .status()?;
        if !status.success() {
            anyhow::bail!("command failed: {}", cmd);
        }
    }
    Ok(())
}
