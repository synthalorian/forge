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
            std::path::Path::new("/home/synth/llama.cpp/llama-swap/config.yaml").exists(),
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
