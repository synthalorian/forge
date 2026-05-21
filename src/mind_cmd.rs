//! CLI command handlers for the Mind module (forge breathe, forge strike).
//!
//! Provides themed terminal output for agent status, model listing,
//! and task execution through the AI agent harness.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

use crate::cli::{BreatheAction, BreatheArgs, StrikeArgs};
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
            crate::mind::ServiceStatus::Running => {
                crate::theme::style_success("●", theme)
            }
            crate::mind::ServiceStatus::Stopped => {
                crate::theme::style_error("○", theme)
            }
            crate::mind::ServiceStatus::NotInstalled => {
                crate::theme::style_muted("○", theme)
            }
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
        crate::theme::style_border(
            "────────────────────────────────",
            theme
        )
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
        crate::theme::style_border(
            "────────────────────────────────",
            theme
        )
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
    println!(
        "  {}",
        crate::theme::style_border(&"─".repeat(50), theme)
    );

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
