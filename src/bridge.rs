//! Bridge module — Phase 3: Connection status, webhooks, notifications.
//!
//! The Bridge links everything together.

use anyhow::Result;
use std::os::unix::fs::PermissionsExt;

use crate::cli::{BridgeAction, BridgeArgs};
use crate::config::Config;

pub fn run_bridge(cfg: &Config, args: &BridgeArgs) -> Result<()> {
    match &args.action {
        Some(BridgeAction::Status) | None => run_status(cfg),
        Some(BridgeAction::Hooks) => run_hooks(cfg),
        Some(BridgeAction::Notify { channel, message }) => run_notify(cfg, channel, message),
        Some(BridgeAction::Sync { verbose }) => run_sync(cfg, *verbose),
    }
}

fn run_status(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Bridge — Connection Status", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let integrations = [
        ("Forge CLI", true), // We're running it, so it's available
        ("Forge Hub", check_url("http://localhost:3000")),
        ("Git", which_exists("git")),
        ("zstd", which_exists("zstd")),
        ("OpenCode", which_exists("opencode")),
        (
            "llama-swap",
            std::path::Path::new(&std::env::var("LLAMA_SWAP_CONFIG").unwrap_or_else(|_| "/home/synth/llama.cpp/llama-swap/config.yaml".to_string())).exists(),
        ),
        ("Hermes", which_exists("hermes")),
        ("Codex CLI", which_exists("codex")),
        ("ripgrep", which_exists("rg")),
        ("Docker", which_exists("docker")),
        ("Redis", which_exists("redis-cli")),
    ];

    let mut connected = 0u64;
    let mut total = 0u64;

    for (name, available) in &integrations {
        total += 1;
        if *available {
            connected += 1;
            println!(
                "  {} {}",
                crate::theme::style_success("●", theme),
                crate::theme::style_value(name, theme),
            );
        } else {
            println!(
                "  {} {}",
                crate::theme::style_muted("○", theme),
                crate::theme::style_muted(name, theme),
            );
        }
    }

    println!();
    println!(
        "  {} {}/{} connected",
        crate::theme::style_bold_header("Summary:", theme),
        crate::theme::style_value(&connected.to_string(), theme),
        crate::theme::style_value(&total.to_string(), theme),
    );

    Ok(())
}

fn run_hooks(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Bridge — Webhook Endpoints", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Check if hooks directory exists
    let hooks_dir = cfg
        .archive_dir
        .parent()
        .unwrap_or(&cfg.archive_dir)
        .join("scripts");

    if !hooks_dir.exists() {
        println!(
            "  {}",
            crate::theme::style_muted("No hooks directory found.", theme)
        );
        println!(
            "  {}",
            crate::theme::style_muted(
                "Create scripts in ~/.local/share/forge/scripts/ to register hooks.",
                theme
            )
        );
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&hooks_dir)?
        .filter_map(|e| e.ok())
        .collect();

    if entries.is_empty() {
        println!(
            "  {}",
            crate::theme::style_muted("No hooks configured.", theme)
        );
    } else {
        for entry in &entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_exec = entry
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);
            println!(
                "  {} {} {}",
                if is_exec {
                    crate::theme::style_success("●", theme)
                } else {
                    crate::theme::style_error("○", theme)
                },
                crate::theme::style_accent(&name, theme),
                if !is_exec {
                    crate::theme::style_muted("(not executable)", theme)
                } else {
                    crate::theme::style_muted("", theme)
                },
            );
        }
    }

    println!();
    println!(
        "  {}",
        crate::theme::style_muted(
            "Lifecycle hooks: pre-backup, post-backup, post-restore, post-strike",
            theme
        )
    );

    Ok(())
}

fn run_notify(cfg: &Config, channel: &Option<String>, message: &Option<String>) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let channel = channel.as_deref().unwrap_or("desktop");
    let message = message
        .as_deref()
        .unwrap_or("Test notification from Forge 🔨");

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Bridge — Send Notification", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    match channel {
        "desktop" => {
            // Try notify-send (Linux)
            let result = std::process::Command::new("notify-send")
                .args(["Forge", message])
                .status();

            match result {
                Ok(s) if s.success() => {
                    println!(
                        "  {} Desktop notification sent",
                        crate::theme::style_success("✓", theme)
                    );
                }
                _ => {
                    println!(
                        "  {} notify-send not available — falling back to terminal",
                        crate::theme::style_error("⚠", theme)
                    );
                    println!(
                        "  {} {}",
                        crate::theme::style_label("Message:", theme),
                        crate::theme::style_value(message, theme),
                    );
                }
            }
        }
        "telegram" => {
            println!(
                "  {}",
                crate::theme::style_muted(
                    "Telegram notifications require TELEGRAM_BOT_TOKEN env var.",
                    theme
                )
            );
            println!(
                "  {} Message: {}",
                crate::theme::style_label("Would send:", theme),
                crate::theme::style_value(message, theme),
            );
        }
        "discord" => {
            println!(
                "  {}",
                crate::theme::style_muted(
                    "Discord notifications require DISCORD_WEBHOOK_URL env var.",
                    theme
                )
            );
            println!(
                "  {} Message: {}",
                crate::theme::style_label("Would send:", theme),
                crate::theme::style_value(message, theme),
            );
        }
        _ => {
            println!(
                "  {} Unknown channel '{}'. Use: desktop, telegram, discord",
                crate::theme::style_error("✗", theme),
                crate::theme::style_error(channel, theme),
            );
        }
    }

    Ok(())
}

pub fn run_sync(cfg: &Config, verbose: bool) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let _ = verbose;

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Bridge — Task Sync", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // ── Active Schedules ───────────────────────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Scheduled Backups", theme)
    );
    if let Ok(conn) = crate::db::connect(cfg) {
        match crate::db::list_schedules(&conn) {
            Ok(schedules) => {
                let active: Vec<_> = schedules.iter().filter(|s| s.enabled).collect();
                if active.is_empty() {
                    println!(
                        "  {}",
                        crate::theme::style_muted("  No active schedules", theme)
                    );
                } else {
                    for s in &active {
                        let last = s
                            .last_run
                            .map(|t| t.format("%m/%d %H:%M").to_string())
                            .unwrap_or_else(|| "never".to_string());
                        println!(
                            "  {} {} — {} (last: {})",
                            crate::theme::style_success("●", theme),
                            crate::theme::style_value(&s.cron_expression, theme),
                            crate::theme::style_muted(&s.target_path, theme),
                            crate::theme::style_value(&last, theme)
                        );
                    }
                }
            }
            Err(e) => println!(
                "  {} {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_muted(&e.to_string(), theme)
            ),
        }
    }
    println!();

    // ── Recent Backups ─────────────────────────────────────────────
    println!("  {}", crate::theme::style_header("Recent Backups", theme));
    if let Ok(conn) = crate::db::connect(cfg) {
        match conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0) FROM backups",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        ) {
            Ok((count, size)) => {
                println!(
                    "  {} {} backups, {} stored",
                    crate::theme::style_value(&count.to_string(), theme),
                    crate::theme::style_muted("total", theme),
                    crate::theme::style_value(&crate::utils::format_size(size as u64), theme)
                );
                if let Ok(mut stmt) = conn.prepare("SELECT repo_name, created_at, size_bytes FROM backups ORDER BY created_at DESC LIMIT 5") {
                    if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))) {
                        for row in rows.flatten().take(5) {
                            let date_preview: String = row.1.chars().take(10).collect();
                            println!("  {} {} — {} ({})", crate::theme::style_accent("▸", theme), crate::theme::style_value(&row.0, theme), crate::theme::style_muted(&crate::utils::format_size(row.2 as u64), theme), crate::theme::style_muted(&date_preview, theme));
                        }
                    }
                }
            }
            Err(_) => println!("  {}", crate::theme::style_muted("No backups yet", theme)),
        }
    }
    println!();

    // ── Agent Sessions ─────────────────────────────────────────────
    println!("  {}", crate::theme::style_header("Agent Sessions", theme));
    let agent_db = cfg
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"))
        .join("agents.db");
    if agent_db.exists() {
        if let Ok(conn) = rusqlite::Connection::open(&agent_db) {
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM agent_sessions", [], |r| {
                r.get::<_, i64>(0)
            }) {
                let latest = conn.query_row("SELECT agent_name, updated_at FROM agent_sessions ORDER BY updated_at DESC LIMIT 1", [], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
                println!(
                    "  {} {} session(s)",
                    crate::theme::style_value(&count.to_string(), theme),
                    crate::theme::style_muted("total", theme)
                );
                if let Ok((agent, updated)) = latest {
                    let date_preview: String = updated.chars().take(10).collect();
                    println!(
                        "  {} {} — last active {}",
                        crate::theme::style_accent("▸", theme),
                        crate::theme::style_value(&agent, theme),
                        crate::theme::style_muted(&date_preview, theme)
                    );
                }
            }
        }
    } else {
        println!(
            "  {}",
            crate::theme::style_muted("No agent sessions yet", theme)
        );
    }
    println!();

    // ── Available Agents ──────────────────────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Available Agents", theme)
    );
    let agents = crate::mind::detect_agents();
    let running: Vec<_> = agents
        .iter()
        .filter(|a| a.status == crate::mind::ServiceStatus::Running)
        .collect();
    let stopped: Vec<_> = agents
        .iter()
        .filter(|a| a.status == crate::mind::ServiceStatus::Stopped)
        .collect();
    for a in &running {
        println!(
            "  {} {} {}",
            crate::theme::style_success("●", theme),
            crate::theme::style_value(&a.name, theme),
            crate::theme::style_muted("running", theme)
        );
    }
    for a in &stopped {
        println!(
            "  {} {} {}",
            crate::theme::style_muted("○", theme),
            crate::theme::style_value(&a.name, theme),
            crate::theme::style_muted("stopped", theme)
        );
    }
    println!();

    // ── Disk & Storage ─────────────────────────────────────────────
    println!("  {}", crate::theme::style_header("Storage", theme));
    let disk = crate::tongs::safe_command("df -h / 2>/dev/null | tail -1");
    let parts: Vec<&str> = disk.split_whitespace().collect();
    if parts.len() >= 5 {
        println!(
            "  {} {} used of {} ({} free)",
            crate::theme::style_value(parts[4], theme),
            crate::theme::style_label("disk:", theme),
            crate::theme::style_value(parts[1], theme),
            crate::theme::style_value(parts[3], theme)
        );
    }
    let forge_size = std::process::Command::new("du")
        .args(["-sh", &cfg.archive_dir.parent().unwrap_or(&cfg.archive_dir).to_string_lossy()])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.split_whitespace().next().unwrap_or("").to_string()
        })
        .unwrap_or_default();
    if !forge_size.trim().is_empty() {
        println!(
            "  {} {}",
            crate::theme::style_label("forge data:", theme),
            crate::theme::style_value(forge_size.trim(), theme)
        );
    }
    println!();

    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    println!(
        "  {} {}",
        crate::theme::style_success("✓", theme),
        crate::theme::style_muted("Sync complete — all task state gathered", theme)
    );
    Ok(())
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_url(url: &str) -> bool {
    std::process::Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--max-time",
            "2",
            url,
        ])
        .output()
        .map(|o| {
            let code = String::from_utf8_lossy(&o.stdout);
            code.trim().starts_with('2') || code.trim().starts_with('3')
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_exists_git() {
        // Git should be available in the test environment
        assert!(which_exists("git"));
    }

    #[test]
    fn which_not_exists_fake() {
        assert!(!which_exists("nonexistent_command_xyz_12345"));
    }
}
