use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

use crate::cli::{ScheduleAction, ScheduleArgs};
use crate::config::Config;
use crate::models::ScheduleConfig;

/// Validate a cron expression has the basic 5-field format (min hour day month weekday).
fn validate_cron(expr: &str) -> Result<()> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        anyhow::bail!(
            "Invalid cron expression '{}': expected 5 fields (min hour day month weekday), got {}",
            expr,
            parts.len()
        );
    }
    Ok(())
}

/// Write a crontab file to `~/.config/forge/crontab` containing entries for all schedules.
fn regenerate_crontab(conn: &Connection, theme: &crate::theme::Theme) -> Result<()> {
    let schedules = crate::db::list_schedules(conn)?;

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/forge"))
        .join("forge");
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create config dir {}", config_dir.display()))?;

    let crontab_path = config_dir.join("crontab");

    if schedules.is_empty() {
        if crontab_path.exists() {
            fs::remove_file(&crontab_path).with_context(|| {
                format!("Failed to remove crontab file {}", crontab_path.display())
            })?;
        }
        println!("{}", crate::theme::style_muted("No schedules remaining — crontab file removed.", theme));
        return Ok(());
    }

    let mut content =
        String::from("# Forge backup schedules — managed by forge. Do not edit manually.\n");

    for schedule in &schedules {
        if schedule.enabled {
            content.push_str(&format!(
                "{} forge backup --path {}\n",
                schedule.cron_expression, schedule.target_path
            ));
        }
    }

    fs::write(&crontab_path, &content)
        .with_context(|| format!("Failed to write crontab file {}", crontab_path.display()))?;

    println!(
        "{} {}",
        crate::theme::style_info("Crontab written to", theme),
        crate::theme::style_value(&crontab_path.display().to_string(), theme),
    );
    println!(
        "  {} crontab {}",
        crate::theme::style_muted("Install with:", theme),
        crate::theme::style_value(&crontab_path.display().to_string(), theme),
    );

    Ok(())
}

/// Run the schedule command.
///
/// Manages cron-based backup schedules stored in the SQLite database.
/// Schedules are applied by generating a crontab file at `~/.config/forge/crontab`
/// that the user can manually install.
pub fn run(cfg: &Config, args: &ScheduleArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let action = args.action.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No schedule action specified. Use add, remove, or list.")
    })?;

    match action {
        ScheduleAction::Add { cron, path } => {
            validate_cron(cron).with_context(|| format!("Invalid cron expression: {cron}"))?;

            if !Path::new(path).exists() {
                anyhow::bail!("Target path does not exist: {path}");
            }

            let conn = crate::db::connect(cfg)?;

            let schedule = ScheduleConfig {
                id: 0,
                cron_expression: cron.clone(),
                target_path: path.clone(),
                enabled: true,
                last_run: None,
                created_at: chrono::Utc::now(),
            };

            let id = crate::db::insert_schedule(&conn, &schedule)
                .context("Failed to insert schedule")?;

            regenerate_crontab(&conn, theme)?;

            println!(
                "{} {} {} {} {} {}",
                crate::theme::style_success("Schedule added", theme),
                crate::theme::style_muted("(id=", theme),
                crate::theme::style_accent(&id.to_string(), theme),
                crate::theme::style_muted("):", theme),
                crate::theme::style_value(cron, theme),
                crate::theme::style_muted(&format!("→ {path}"), theme),
            );
        }
        ScheduleAction::Remove { id } => {
            let conn = crate::db::connect(cfg)?;

            crate::db::delete_schedule(&conn, *id)?;

            regenerate_crontab(&conn, theme)?;

            println!(
                "{} {}",
                crate::theme::style_success("Schedule removed", theme),
                crate::theme::style_accent(&format!("(id={id})", ), theme),
            );
        }
        ScheduleAction::List => {
            let conn = crate::db::connect(cfg)?;
            let schedules = crate::db::list_schedules(&conn)?;

            if schedules.is_empty() {
                println!("{}", crate::theme::style_muted("No schedules configured", theme));
                return Ok(());
            }

            println!(
                "{}",
                crate::theme::style_header(
                    &format!("{:<5} {:<20} {:<30} {:<8} Last Run", "ID", "Cron", "Target Path", "Enabled"),
                    theme,
                ),
            );
            println!("{}", crate::theme::style_border(&"─".repeat(80), theme));
            for s in &schedules {
                let last_run = s
                    .last_run
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "never".to_string());
                println!(
                    "{} {} {} {} {}",
                    crate::theme::style_accent(&format!("{:<5}", s.id), theme),
                    crate::theme::style_value(&format!("{:<20}", truncate_str(&s.cron_expression, 20)), theme),
                    crate::theme::style_value(&format!("{:<30}", truncate_str(&s.target_path, 30)), theme),
                    crate::theme::style_value(
                        &format!("{:<8}", if s.enabled { "yes" } else { "no" }),
                        theme,
                    ),
                    crate::theme::style_value(&last_run, theme),
                );
            }
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len - 1).collect();
        truncated.push('…');
        truncated
    }
}
