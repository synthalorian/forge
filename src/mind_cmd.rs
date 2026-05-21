//! CLI command handlers for the Mind module (forge breathe, forge strike).
//!
//! Provides themed terminal output for agent status, model listing,
//! and task execution through the AI agent harness.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

use crate::cli::{BreatheAction, BreatheArgs, SessionAction, StrikeArgs};
use crate::config::Config;

/// Run the `forge breathe` command — agent status dashboard.
pub fn run_breathe(cfg: &Config, args: &BreatheArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    match &args.action {
        Some(BreatheAction::Status) | None => breathe_status(theme),
        Some(BreatheAction::Models) => breathe_models(theme),
        Some(BreatheAction::Vault) => breathe_vault(theme),
        Some(BreatheAction::Prompts) => breathe_prompts(theme),
        Some(BreatheAction::Pipe { path }) => run_pipe(cfg, path),
        Some(BreatheAction::Sessions { action }) => run_sessions(cfg, action),
    }
}

/// Run the `forge strike <task>` command — execute via AI agent.
pub fn run_strike(cfg: &Config, args: &StrikeArgs) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    let agents = crate::mind::detect_agents();
    let routing = crate::mind::route_task(&args.task, &agents);
    let target_agent = args.agent.as_deref().unwrap_or(&routing.agent);

    println!();
    println!(
        "  {} {}",
        crate::theme::style_bold_header("⚡ Strike", theme),
        crate::theme::style_accent(&args.task, theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "────────────────────────────────────────────────────────────",
            theme
        )
    );
    println!(
        "  {} {} {} ({})",
        crate::theme::style_label("Agent:", theme),
        crate::theme::style_value(target_agent, theme),
        crate::theme::style_muted(
            if args.agent.is_some() {
                "(forced)"
            } else {
                "(auto)"
            },
            theme
        ),
        crate::theme::style_info(&format!("{}", routing.agent_type), theme),
    );
    println!();

    match crate::mind::execute_strike(&args.task, Some(target_agent)) {
        Ok(output) => {
            if !output.is_empty() {
                for line in output.lines() {
                    println!("  {}", crate::theme::style_accent(line, theme));
                }
            }
            println!();
            println!(
                "  {} Task completed",
                crate::theme::style_success("✓", theme),
            );
        }
        Err(e) => {
            println!(
                "  {} {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_error(&e.to_string(), theme),
            );
            println!(
                "  {} Try running {} first to check agent availability.",
                crate::theme::style_muted("ℹ", theme),
                crate::theme::style_value("forge breathe", theme),
            );
        }
    }

    println!();
    Ok(())
}

fn breathe_status(theme: &crate::theme::Theme) -> Result<()> {
    let agents = crate::mind::detect_agents();
    let running_count = agents
        .iter()
        .filter(|a| a.status == crate::mind::ServiceStatus::Running)
        .count();

    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("🧠 Agent Status", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "────────────────────────────────────────────────────────────",
            theme
        )
    );

    for agent in &agents {
        let status_indicator = match agent.status {
            crate::mind::ServiceStatus::Running => crate::theme::style_success("●", theme),
            crate::mind::ServiceStatus::Stopped => crate::theme::style_error("○", theme),
            crate::mind::ServiceStatus::NotInstalled => crate::theme::style_muted("○", theme),
        };

        let status_text = match agent.status {
            crate::mind::ServiceStatus::Running => {
                crate::theme::style_success(&agent.status.to_string(), theme)
            }
            crate::mind::ServiceStatus::Stopped => {
                crate::theme::style_error(&agent.status.to_string(), theme)
            }
            crate::mind::ServiceStatus::NotInstalled => {
                crate::theme::style_muted(&agent.status.to_string(), theme)
            }
        };

        let type_info = crate::theme::style_muted(&format!("{}", agent.agent_type), theme);

        let extra = match (&agent.model, &agent.version) {
            (Some(model), Some(version)) => {
                format!(
                    " {} {} {} {}",
                    crate::theme::style_muted("│", theme),
                    crate::theme::style_info(model, theme),
                    crate::theme::style_muted("│", theme),
                    crate::theme::style_value(version, theme),
                )
            }
            (Some(model), None) => format!(
                " {} {}",
                crate::theme::style_muted("│", theme),
                crate::theme::style_info(model, theme),
            ),
            (None, Some(version)) => format!(
                " {} {}",
                crate::theme::style_muted("│", theme),
                crate::theme::style_value(version, theme),
            ),
            (None, None) => String::new(),
        };

        println!(
            "  {} {:<14} {} {}{}",
            status_indicator,
            crate::theme::style_accent(&agent.name, theme),
            status_text,
            type_info,
            extra,
        );
    }

    println!(
        "  {}",
        crate::theme::style_border(
            "────────────────────────────────────────────────────────────",
            theme
        )
    );
    println!(
        "  {} {} agent(s) available",
        crate::theme::style_muted("ℹ", theme),
        crate::theme::style_value(&running_count.to_string(), theme),
    );
    println!(
        "  {} Use {} to delegate a task",
        crate::theme::style_muted("Tip:", theme),
        crate::theme::style_value("forge strike <task>", theme),
    );
    println!();

    Ok(())
}

fn breathe_models(theme: &crate::theme::Theme) -> Result<()> {
    let models = crate::mind::detect_models();

    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("🤖 Available Models", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "──────────────────────────────────────────────────────",
            theme
        )
    );

    if models.is_empty() {
        println!(
            "  {} No local models found. Check your llama-swap configuration.",
            crate::theme::style_muted("ℹ", theme),
        );
    } else {
        for model in &models {
            println!(
                "  {} {} {}",
                crate::theme::style_accent("▸", theme),
                crate::theme::style_value(&model.name, theme),
                crate::theme::style_muted(&format!("({})", model.provider), theme),
            );
        }
    }

    println!();
    Ok(())
}

fn breathe_vault(theme: &crate::theme::Theme) -> Result<()> {
    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("🔐 Credential Vault", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("────────────────────────────────", theme)
    );
    println!(
        "  {} Coming in a future release.",
        crate::theme::style_muted("⏳", theme),
    );
    println!(
        "  {} Will support OAuth tokens, API keys, and credential rotation.",
        crate::theme::style_muted("ℹ", theme),
    );
    println!();
    Ok(())
}

fn breathe_prompts(theme: &crate::theme::Theme) -> Result<()> {
    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("📝 Prompt Library", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border("────────────────────────────────", theme)
    );
    println!(
        "  {} Coming in a future release.",
        crate::theme::style_muted("⏳", theme),
    );
    println!(
        "  {} Will support TOML-based prompt CRUD at {}.",
        crate::theme::style_muted("ℹ", theme),
        crate::theme::style_value("~/.forge/prompts/", theme),
    );
    println!();
    Ok(())
}

// ── Pipeline System ──────────────────────────────────────────

/// TOML structure for a multi-step agent pipeline definition.
#[derive(Debug, Clone, Deserialize)]
struct PipelineDef {
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    steps: Vec<PipelineStep>,
}

/// A single step in a pipeline.
#[derive(Debug, Clone, Deserialize)]
struct PipelineStep {
    name: String,
    task: String,
    agent: Option<String>,
    #[serde(default)]
    input_keys: Vec<String>,
}

fn run_pipe(cfg: &Config, path: &str) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read pipeline file: {path}"))?;
    let pipeline: PipelineDef = toml::from_str(&content)
        .with_context(|| format!("Failed to parse pipeline TOML: {path}"))?;

    if pipeline.steps.is_empty() {
        anyhow::bail!("Pipeline has no steps defined");
    }

    let name_display = pipeline.name.as_deref().unwrap_or("unnamed");
    let total = pipeline.steps.len();

    println!();
    println!(
        "  {} {}",
        crate::theme::style_bold_header("🔧 Pipeline:", theme),
        crate::theme::style_accent(name_display, theme)
    );
    if let Some(desc) = &pipeline.description {
        println!("  {}", crate::theme::style_muted(desc, theme));
    }
    println!(
        "  {} {}",
        crate::theme::style_value(&total.to_string(), theme),
        crate::theme::style_muted("step(s)", theme)
    );
    println!("  {}", crate::theme::style_border(&"─".repeat(50), theme));

    let mut outputs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for (i, step) in pipeline.steps.iter().enumerate() {
        let step_num = i + 1;

        let mut context_lines: Vec<&str> = Vec::new();
        for key in &step.input_keys {
            if let Some(val) = outputs.get(key) {
                if !val.trim().is_empty() {
                    context_lines.push(val.trim());
                }
            }
        }
        let combined_context = if context_lines.is_empty() {
            String::new()
        } else {
            format!(
                "\nContext from previous steps:\n{}\n",
                context_lines.join("\n---\n")
            )
        };

        let full_task = if combined_context.is_empty() {
            step.task.clone()
        } else {
            format!("{}\n{}", combined_context, step.task)
        };

        let task_preview: String = step.task.chars().take(60).collect();

        println!();
        println!(
            "  {} {} {} {}",
            crate::theme::style_label(&format!("[{step_num}/{total}]"), theme),
            crate::theme::style_bold_header(&step.name, theme),
            crate::theme::style_muted("→", theme),
            crate::theme::style_info(&task_preview, theme),
        );
        if let Some(ref agent) = step.agent {
            println!(
                "  {} {}",
                crate::theme::style_muted("Agent:", theme),
                crate::theme::style_value(agent, theme),
            );
        }
        if !step.input_keys.is_empty() {
            println!(
                "  {} {}",
                crate::theme::style_muted("Inputs:", theme),
                crate::theme::style_value(&step.input_keys.join(", "), theme),
            );
        }

        let target_agent = step.agent.as_deref();
        match crate::mind::execute_strike(&full_task, target_agent) {
            Ok(output) => {
                outputs.insert(step.name.clone(), output.clone());
                println!(
                    "  {} {}",
                    crate::theme::style_success("✓", theme),
                    crate::theme::style_muted("Completed", theme),
                );
                if output.len() < 200 {
                    println!("  {}", crate::theme::style_value(&output, theme));
                } else {
                    let preview: String = output.chars().take(200).collect();
                    println!(
                        "  {}{}",
                        crate::theme::style_value(&preview, theme),
                        crate::theme::style_muted("… (truncated)", theme),
                    );
                }
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    crate::theme::style_error("✗", theme),
                    crate::theme::style_error(&e.to_string(), theme),
                );
                anyhow::bail!("Pipeline step '{}' failed: {}", step.name, e);
            }
        }
    }

    println!();
    println!(
        "  {} {}",
        crate::theme::style_bold_header("✓ Pipeline complete:", theme),
        crate::theme::style_accent(name_display, theme),
    );
    println!(
        "  {} {}",
        crate::theme::style_value(&total.to_string(), theme),
        crate::theme::style_muted("step(s) executed successfully", theme),
    );
    println!();

    Ok(())
}

fn run_sessions(cfg: &Config, action: &Option<SessionAction>) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = connect_session_db(cfg)?;
    match action {
        Some(SessionAction::List { agent: _, limit }) => {
            let l = limit.unwrap_or(20);
            let s = list_sessions_internal(&conn, l)?;
            print_sessions_internal(&s, theme);
        }
        None => {
            let s = list_sessions_internal(&conn, 20)?;
            print_sessions_internal(&s, theme);
        }
        Some(SessionAction::Show { id }) => {
            let s = get_session_internal(&conn, *id)?;
            let msgs = get_messages_internal(&conn, *id)?;
            println!();
            println!(
                "  #{} {}",
                theme_style_value(&s.0.to_string(), theme),
                theme_style_accent(&s.1, theme)
            );
            println!("  {}", theme_style_border(&"-".repeat(50), theme));
            for (r, c) in &msgs {
                let p = c.chars().take(100).collect::<String>();
                let tag = if r == "user" { "U" } else { "A" };
                println!(
                    "  {} {}",
                    theme_style_value(tag, theme),
                    theme_style_value(&p, theme)
                );
            }
            println!();
        }
        Some(SessionAction::Delete { id }) => {
            delete_session_internal(&conn, *id)?;
            println!(
                "  \u{2713} Session #{} deleted",
                theme_style_value(&id.to_string(), theme)
            );
        }
        Some(SessionAction::Create { agent, message }) => {
            let id = create_session_internal(&conn, agent, message.as_deref())?;
            println!(
                "  \u{2713} Session #{} for {}",
                theme_style_value(&id.to_string(), theme),
                theme_style_accent(agent, theme)
            );
        }
    }
    Ok(())
}

fn print_sessions_internal(s: &[(i64, String, i64, String)], theme: &crate::theme::Theme) {
    println!();
    println!("  {}", theme_style_bold_header("Agent Sessions", theme));
    println!("  {}", theme_style_border(&"-".repeat(50), theme));
    if s.is_empty() {
        println!("  {}", theme_style_muted("No sessions found.", theme));
    } else {
        for (id, name, count, summary) in s {
            let p = summary.chars().take(40).collect::<String>();
            println!(
                "  {} {} {} {}",
                theme_style_value(&id.to_string(), theme),
                theme_style_accent(name, theme),
                theme_style_value(&count.to_string(), theme),
                theme_style_muted(&p, theme)
            );
        }
    }
    println!();
}

fn theme_style_value(t: &str, theme: &crate::theme::Theme) -> crate::theme::StyledString {
    crate::theme::style_value(t, theme)
}
fn theme_style_accent(t: &str, theme: &crate::theme::Theme) -> crate::theme::StyledString {
    crate::theme::style_accent(t, theme)
}
fn theme_style_muted(t: &str, theme: &crate::theme::Theme) -> crate::theme::StyledString {
    crate::theme::style_muted(t, theme)
}
fn theme_style_border(t: &str, theme: &crate::theme::Theme) -> crate::theme::StyledString {
    crate::theme::style_border(t, theme)
}
fn theme_style_bold_header(t: &str, theme: &crate::theme::Theme) -> crate::theme::StyledString {
    crate::theme::style_bold_header(t, theme)
}

fn connect_session_db(cfg: &Config) -> Result<rusqlite::Connection> {
    let p = cfg
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"))
        .join("agents.db");
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = rusqlite::Connection::open(&p)?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS agent_sessions (id INTEGER PRIMARY KEY AUTOINCREMENT, agent_name TEXT NOT NULL, summary TEXT NOT NULL DEFAULT '', message_count INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL, updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS session_messages (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id INTEGER NOT NULL REFERENCES agent_sessions(id), role TEXT NOT NULL, content TEXT NOT NULL, created_at TEXT NOT NULL);")?;
    Ok(conn)
}

fn create_session_internal(
    conn: &rusqlite::Connection,
    agent: &str,
    msg: Option<&str>,
) -> Result<i64> {
    let now = chrono::Utc::now().to_rfc3339();
    let summary = msg
        .map(|m| m.chars().take(100).collect::<String>())
        .unwrap_or_default();
    conn.execute("INSERT INTO agent_sessions (agent_name, summary, message_count, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)", (agent, &summary, if msg.is_some() { 1 } else { 0 }, &now, &now))?;
    let id = conn.last_insert_rowid();
    if let Some(m) = msg {
        conn.execute("INSERT INTO session_messages (session_id, role, content, created_at) VALUES (?1, 'user', ?2, ?3)", (id, m, &now))?;
    }
    Ok(id)
}

fn list_sessions_internal(
    conn: &rusqlite::Connection,
    limit: usize,
) -> Result<Vec<(i64, String, i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, agent_name, message_count, summary FROM agent_sessions ORDER BY updated_at DESC LIMIT ?1")?;
    let rows = stmt.query_map([limit as i64], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?;
    let mut v = Vec::new();
    for row in rows {
        v.push(row?);
    }
    Ok(v)
}

fn get_session_internal(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<(i64, String, i64, String)> {
    conn.query_row(
        "SELECT id, agent_name, message_count, summary FROM agent_sessions WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )
    .with_context(|| format!("Session {id} not found"))
}

fn get_messages_internal(conn: &rusqlite::Connection, id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn
        .prepare("SELECT role, content FROM session_messages WHERE session_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut v = Vec::new();
    for row in rows {
        v.push(row?);
    }
    Ok(v)
}

fn delete_session_internal(conn: &rusqlite::Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM session_messages WHERE session_id = ?1", [id])?;
    let n = conn.execute("DELETE FROM agent_sessions WHERE id = ?1", [id])?;
    if n == 0 {
        anyhow::bail!("Session {id} not found");
    }
    Ok(())
}
