//! Tongs module — Phase 2D: System resource dashboard, dotfile tracking, diagnostics.
//!
//! The Tongs grip your environment. This module provides:
//! - `forge grip` — System resource dashboard (CPU, memory, disk, GPU, services)
//! - `forge grip dotfiles` — Track and version dotfiles
//! - `forge grip diagnose` — System health check

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::{DotfilesAction, GripAction, GripArgs};
use crate::config::Config;

/// Run the grip subcommand.
pub fn run_grip(cfg: &Config, args: &GripArgs) -> Result<()> {
    match &args.action {
        Some(GripAction::Dashboard) | None => run_dashboard(cfg),
        Some(GripAction::Diagnose) => run_diagnose(cfg),
        Some(GripAction::Dotfiles { action }) => run_dotfiles(cfg, action),
    }
}

// ── System Dashboard ────────────────────────────────────────────────

pub fn run_dashboard(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Grip — System Dashboard", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Hostname & kernel
    let hostname = safe_command("hostname");
    let kernel = safe_command("uname -r");
    let os_name =
        safe_command("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2");
    let uptime = safe_command("uptime -p 2>/dev/null || uptime")
        .trim()
        .trim_start_matches("up ")
        .to_string();

    println!(
        "  {} {} on {} ({})",
        crate::theme::style_label("Host:", theme),
        crate::theme::style_value(&hostname, theme),
        crate::theme::style_accent(&os_name, theme),
        crate::theme::style_muted(&format!("kernel {}", kernel.trim()), theme),
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Uptime:", theme),
        crate::theme::style_value(&uptime, theme),
    );
    println!();

    // CPU
    let cpu_model = safe_command("lscpu 2>/dev/null | grep 'Model name' | cut -d: -f2");
    let cpu_cores = safe_command("nproc");
    let load_avg = safe_command("cat /proc/loadavg 2>/dev/null")
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(", ");

    println!("  {}", crate::theme::style_header("CPU", theme));
    println!(
        "    {} {}",
        crate::theme::style_label("Model:", theme),
        crate::theme::style_value(cpu_model.trim(), theme),
    );
    println!(
        "    {} {} | {} {}",
        crate::theme::style_label("Cores:", theme),
        crate::theme::style_value(cpu_cores.trim(), theme),
        crate::theme::style_label("Load:", theme),
        crate::theme::style_value(&load_avg, theme),
    );
    println!();

    // Memory
    let mem_info = parse_memory_info();
    println!("  {}", crate::theme::style_header("Memory", theme));
    println!("    {}", crate::theme::style_value(&mem_info, theme),);
    println!();

    // Disk
    let disk_info = parse_disk_info();
    println!("  {}", crate::theme::style_header("Disk", theme));
    for line in disk_info.lines() {
        println!("    {}", crate::theme::style_value(line, theme));
    }
    println!();

    // Running dev services
    println!(
        "  {}",
        crate::theme::style_header("Running Services", theme)
    );
    let services = detect_services();
    if services.is_empty() {
        println!(
            "    {}",
            crate::theme::style_muted("No dev services detected", theme)
        );
    } else {
        for svc in &services {
            println!(
                "    {} {}",
                crate::theme::style_success("●", theme),
                crate::theme::style_value(svc, theme),
            );
        }
    }

    // Forge stats
    println!();
    println!("  {}", crate::theme::style_header("Forge", theme));
    if cfg.db_path.exists() {
        if let Ok(conn) = crate::db::connect(cfg) {
            let backup_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM backups", [], |row| row.get(0))
                .unwrap_or(0);
            let repo_count: i64 = conn
                .query_row("SELECT COUNT(DISTINCT repo_name) FROM backups", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            let schedule_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM schedules WHERE enabled = 1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            println!(
                "    {} {} backups, {} repos, {} schedules",
                crate::theme::style_success("●", theme),
                crate::theme::style_value(&backup_count.to_string(), theme),
                crate::theme::style_value(&repo_count.to_string(), theme),
                crate::theme::style_value(&schedule_count.to_string(), theme),
            );
        }
    } else {
        println!(
            "    {}",
            crate::theme::style_muted("Forge not initialized", theme)
        );
    }

    Ok(())
}

// ── Diagnostics ─────────────────────────────────────────────────────

pub fn run_diagnose(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Grip Diagnose — System Health", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let mut issues = 0u64;

    // Check disk space
    let disk_output = safe_command("df -h / 2>/dev/null");
    let disk_percent = disk_output
        .lines()
        .nth(1)
        .and_then(|l| l.split_whitespace().nth(4))
        .and_then(|p| p.trim_end_matches('%').parse::<u64>().ok())
        .unwrap_or(0);

    if disk_percent > 90 {
        println!(
            "  {} Disk usage at {}% — critically low!",
            crate::theme::style_error("⚠", theme),
            crate::theme::style_error(&disk_percent.to_string(), theme),
        );
        issues += 1;
    } else if disk_percent > 75 {
        println!(
            "  {} Disk usage at {}% — consider cleanup",
            crate::theme::style_error("⚠", theme),
            crate::theme::style_value(&disk_percent.to_string(), theme),
        );
        issues += 1;
    } else {
        println!(
            "  {} Disk usage: {}% — OK",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&disk_percent.to_string(), theme),
        );
    }

    // Check memory
    let mem_output = safe_command("free 2>/dev/null");
    let mem_percent = mem_output
        .lines()
        .nth(2)
        .and_then(|l| l.split_whitespace().nth(2))
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(0);
    let mem_total = mem_output
        .lines()
        .nth(1)
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(1);

    let mem_usage_pct = ((mem_percent as f64 / mem_total as f64) * 100.0) as u64;
    if mem_usage_pct > 90 {
        println!(
            "  {} Memory usage: {}% — critically high!",
            crate::theme::style_error("⚠", theme),
            crate::theme::style_error(&mem_usage_pct.to_string(), theme),
        );
        issues += 1;
    } else {
        println!(
            "  {} Memory usage: {}%",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&mem_usage_pct.to_string(), theme),
        );
    }

    // Check essential tools
    for tool in &["git", "zstd", "tar"] {
        let found = safe_command(&format!("which {}", tool));
        if found.trim().is_empty() {
            println!(
                "  {} {} not found — required for backups",
                crate::theme::style_error("✗", theme),
                crate::theme::style_error(tool, theme),
            );
            issues += 1;
        } else {
            println!(
                "  {} {} found",
                crate::theme::style_success("✓", theme),
                crate::theme::style_value(tool, theme),
            );
        }
    }

    // Check forge config
    if !cfg.db_path.exists() {
        println!(
            "  {} Forge database not found — run 'forge init'",
            crate::theme::style_error("✗", theme),
        );
        issues += 1;
    } else {
        println!(
            "  {} Forge database: {}",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&cfg.db_path.display().to_string(), theme),
        );
    }

    // Check archive dir
    if cfg.archive_dir.exists() {
        println!(
            "  {} Archive directory: {}",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&cfg.archive_dir.display().to_string(), theme),
        );
    } else {
        println!(
            "  {} Archive directory missing: {}",
            crate::theme::style_error("✗", theme),
            crate::theme::style_error(&cfg.archive_dir.display().to_string(), theme),
        );
        issues += 1;
    }

    println!();
    if issues == 0 {
        println!(
            "  {}",
            crate::theme::style_success("All systems nominal. The forge burns bright. 🔨", theme)
        );
    } else {
        println!(
            "  {} {}",
            crate::theme::style_error("⚠", theme),
            crate::theme::style_error(&format!("{} issue(s) found — see above", issues), theme),
        );
    }

    Ok(())
}

// ── Dotfile Tracking ────────────────────────────────────────────────

fn dotfiles_dir(cfg: &Config) -> PathBuf {
    cfg.archive_dir
        .parent()
        .unwrap_or(&cfg.archive_dir)
        .join("dotfiles")
}

pub fn run_dotfiles(cfg: &Config, action: &Option<DotfilesAction>) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let dir = dotfiles_dir(cfg);

    match action {
        None | Some(DotfilesAction::List) => {
            println!(
                "{}",
                crate::theme::style_bold_header("Forge Grip Dotfiles — Tracked Files", theme)
            );
            println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

            if !dir.exists() {
                println!(
                    "  {}",
                    crate::theme::style_muted("No dotfiles tracked yet.", theme)
                );
                println!(
                    "  {}",
                    crate::theme::style_muted(
                        "Use 'forge grip dotfiles track <path>' to start.",
                        theme
                    )
                );
                return Ok(());
            }

            let manifest_path = dir.join("manifest.json");
            if !manifest_path.exists() {
                println!(
                    "  {}",
                    crate::theme::style_muted("No manifest found.", theme)
                );
                return Ok(());
            }

            let manifest = fs::read_to_string(&manifest_path)?;
            let entries: Vec<serde_json::Value> = serde_json::from_str(&manifest)?;

            if entries.is_empty() {
                println!(
                    "  {}",
                    crate::theme::style_muted("No dotfiles tracked.", theme)
                );
            } else {
                for entry in &entries {
                    let name = entry["name"].as_str().unwrap_or("?");
                    let source = entry["source"].as_str().unwrap_or("?");
                    let versions = entry["versions"].as_u64().unwrap_or(0);
                    println!(
                        "  {} {} {} {}",
                        crate::theme::style_success("●", theme),
                        crate::theme::style_accent(name, theme),
                        crate::theme::style_muted(&format!("({})", source), theme),
                        crate::theme::style_value(&format!("{} versions", versions), theme),
                    );
                }
            }
        }
        Some(DotfilesAction::Track { path }) => {
            let source = PathBuf::from(path);
            if !source.exists() {
                anyhow::bail!("File not found: {}", path);
            }

            fs::create_dir_all(&dir)?;
            let manifest_path = dir.join("manifest.json");

            let mut entries: Vec<serde_json::Value> = if manifest_path.exists() {
                let content = fs::read_to_string(&manifest_path)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                Vec::new()
            };

            let name = source
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let source_str = source.to_string_lossy().to_string();

            // Check if already tracked
            if let Some(existing) = entries.iter_mut().find(|e| e["source"] == source_str) {
                // Update version count
                let versions = existing["versions"].as_u64().unwrap_or(0) + 1;
                existing["versions"] = serde_json::json!(versions);
                // Copy current version
                let versioned = dir.join(format!("{}.v{}", name, versions));
                fs::copy(&source, &versioned)?;
                println!(
                    "  {} Updated {} — now {} versions",
                    crate::theme::style_success("✓", theme),
                    crate::theme::style_accent(&name, theme),
                    crate::theme::style_value(&versions.to_string(), theme),
                );
            } else {
                // New entry
                let versioned = dir.join(format!("{}.v1", name));
                fs::copy(&source, &versioned)?;
                entries.push(serde_json::json!({
                    "name": name,
                    "source": source_str,
                    "versions": 1
                }));
                println!(
                    "  {} Now tracking {}",
                    crate::theme::style_success("✓", theme),
                    crate::theme::style_accent(&name, theme),
                );
            }

            let manifest_str = serde_json::to_string_pretty(&entries)?;
            fs::write(&manifest_path, manifest_str)?;
        }
        Some(DotfilesAction::Restore { name }) => {
            if !dir.exists() {
                anyhow::bail!("No dotfiles directory found. Track some files first.");
            }

            let manifest_path = dir.join("manifest.json");
            if !manifest_path.exists() {
                anyhow::bail!("No manifest found.");
            }

            let manifest = fs::read_to_string(&manifest_path)?;
            let entries: Vec<serde_json::Value> = serde_json::from_str(&manifest)?;

            if let Some(target_name) = name {
                let entry = entries
                    .iter()
                    .find(|e| e["name"].as_str() == Some(target_name.as_str()))
                    .ok_or_else(|| {
                        anyhow::anyhow!("Dotfile '{}' not found in manifest", target_name)
                    })?;

                let versions = entry["versions"].as_u64().unwrap_or(0);
                let versioned = dir.join(format!("{}.v{}", target_name, versions));
                let dest = entry["source"].as_str().unwrap_or(".");

                if versioned.exists() {
                    fs::copy(&versioned, dest)?;
                    println!(
                        "  {} Restored {} → {}",
                        crate::theme::style_success("✓", theme),
                        crate::theme::style_accent(target_name, theme),
                        crate::theme::style_value(dest, theme),
                    );
                } else {
                    anyhow::bail!("Versioned file not found: {}", versioned.display());
                }
            } else {
                // Restore all
                for entry in &entries {
                    let name_str = entry["name"].as_str().unwrap_or("?");
                    let versions = entry["versions"].as_u64().unwrap_or(0);
                    let versioned = dir.join(format!("{}.v{}", name_str, versions));
                    let dest = entry["source"].as_str().unwrap_or(".");

                    if versioned.exists() {
                        fs::copy(&versioned, dest)?;
                        println!(
                            "  {} Restored {} → {}",
                            crate::theme::style_success("✓", theme),
                            crate::theme::style_accent(name_str, theme),
                            crate::theme::style_value(dest, theme),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────

fn safe_command(cmd: &str) -> String {
    std::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

fn parse_memory_info() -> String {
    let output = safe_command("free -h 2>/dev/null");
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 3 {
        return "N/A".to_string();
    }
    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 3 {
        return "N/A".to_string();
    }
    format!("{} / {} used", parts[2], parts[1])
}

fn parse_disk_info() -> String {
    let output = safe_command("df -h 2>/dev/null | head -5");
    output
        .lines()
        .filter(|l| !l.starts_with("Filesystem"))
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 6 {
                format!(
                    "{} — {} / {} ({} used) on {}",
                    parts[5], parts[2], parts[1], parts[4], parts[0]
                )
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn detect_services() -> Vec<String> {
    let mut services = Vec::new();
    let checks = [
        ("rails server", "pgrep -f 'rails s'"),
        ("rails console", "pgrep -f 'rails c'"),
        ("puma", "pgrep -f puma"),
        ("sidekiq", "pgrep -f sidekiq"),
        ("redis", "pgrep -f redis-server"),
        ("postgres", "pgrep -f postgres"),
        ("nginx", "pgrep -f nginx"),
        ("llama.cpp", "pgrep -f llama"),
        ("opencode", "pgrep -x opencode"),
        ("docker", "pgrep -f dockerd"),
    ];

    for (name, check) in &checks {
        if safe_command(check).trim().parse::<u32>().is_ok()
            || std::process::Command::new("sh")
                .args(["-c", check])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        {
            services.push(name.to_string());
        }
    }

    services
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, RetentionConfig};
    use tempfile::TempDir;

    fn test_config(tmp: &TempDir) -> Config {
        Config {
            archive_dir: tmp.path().join("archives"),
            db_path: tmp.path().join("forge.db"),
            default_compression: 3,
            repo_paths: vec![],
            retention: RetentionConfig {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            theme: "synthwave84".to_string(),
        }
    }

    #[test]
    fn dotfiles_dir_under_forge_data() {
        let tmp = TempDir::new().unwrap();
        let cfg = test_config(&tmp);
        let dir = dotfiles_dir(&cfg);
        assert!(dir.to_string_lossy().contains("dotfiles"));
    }

    #[test]
    fn safe_command_handles_missing_binary() {
        let result = safe_command("nonexistent_command_12345");
        assert!(result.is_empty() || result.trim().is_empty());
    }
}
