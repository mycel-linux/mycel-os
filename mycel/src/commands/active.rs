use anyhow::Result;
use colored::Colorize;
use std::fs;

const GEN_FILE: &str = "/etc/mycel/generation";

pub fn run(gen_only: bool) -> Result<()> {
    let gen = fs::read_to_string(GEN_FILE)
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .to_string();

    if gen_only {
        println!("{}", gen);
        return Ok(());
    }

    let kernel = fs::read_to_string("/proc/version")
        .map(|v| v.split_whitespace().nth(2).unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let uptime = fs::read_to_string("/proc/uptime")
        .map(|u| {
            let secs = u.split_whitespace().next().unwrap_or("0")
                .parse::<f64>().unwrap_or(0.0) as u64;
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            format!("{}h {}m", h, m)
        })
        .unwrap_or_else(|_| "unknown".to_string());

    let services: Vec<String> = fs::read_dir("/run/service")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    println!("{}", "Active System".bold());
    println!("{}", "─────────────────────────────────".dimmed());
    println!("  {}  {}", "generation".blue().bold(), gen);
    println!("  {}      {}", "kernel".blue().bold(), kernel);
    println!("  {}      {}", "uptime".blue().bold(), uptime);
    println!("  {}    {}", "services".blue().bold(), services.join(", "));

    Ok(())
}
