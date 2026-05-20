//! CLI command handlers for the Mind module (forge breathe, forge strike).
//!
//! Provides themed terminal output for agent status, model listing,
//! and task execution through the AI agent harness.

use anyhow::Result;

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
        crate::theme::style_bold_header("\u{26a1} Strike", theme),
        crate::theme::style_accent(&args.task, theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
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
                    println!("  {}", crate::theme::style_accent(line, theme),);
                }
            }
            println!();
            println!(
                "  {} Task completed",
                crate::theme::style_success("\u{2713}", theme),
            );
        }
        Err(e) => {
            println!(
                "  {} {}",
                crate::theme::style_error("\u{2717}", theme),
                crate::theme::style_error(&e.to_string(), theme),
            );
            println!(
                "  {} Try running {} first to check agent availability.",
                crate::theme::style_muted("\u{2139}", theme),
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
        crate::theme::style_bold_header("\u{1f9e0} Agent Status", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            theme
        )
    );

    for agent in &agents {
        let status_indicator = match agent.status {
            crate::mind::ServiceStatus::Running => {
                crate::theme::style_success("\u{25cf}", theme) // ●
            }
            crate::mind::ServiceStatus::Stopped => {
                crate::theme::style_error("\u{25cb}", theme) // ○
            }
            crate::mind::ServiceStatus::NotInstalled => {
                crate::theme::style_muted("\u{25cb}", theme) // ○
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
                    crate::theme::style_muted("\u{2502}", theme),
                    crate::theme::style_info(model, theme),
                    crate::theme::style_muted("\u{2502}", theme),
                    crate::theme::style_value(version, theme),
                )
            }
            (Some(model), None) => format!(
                " {} {}",
                crate::theme::style_muted("\u{2502}", theme),
                crate::theme::style_info(model, theme),
            ),
            (None, Some(version)) => format!(
                " {} {}",
                crate::theme::style_muted("\u{2502}", theme),
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
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            theme
        )
    );
    println!(
        "  {} {} agent(s) available",
        crate::theme::style_muted("\u{2139}", theme),
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
        crate::theme::style_bold_header("\u{1f916} Available Models", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            theme
        )
    );

    if models.is_empty() {
        println!(
            "  {} No local models found. Check your llama-swap configuration.",
            crate::theme::style_muted("\u{2139}", theme),
        );
    } else {
        for model in &models {
            println!(
                "  {} {} {}",
                crate::theme::style_accent("\u{25b8}", theme),
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
        crate::theme::style_bold_header("\u{1f510} Credential Vault", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            theme
        )
    );
    println!(
        "  {} Coming in a future release.",
        crate::theme::style_muted("\u{23f3}", theme),
    );
    println!(
        "  {} Will support OAuth tokens, API keys, and credential rotation.",
        crate::theme::style_muted("\u{2139}", theme),
    );
    println!();
    Ok(())
}

fn breathe_prompts(theme: &crate::theme::Theme) -> Result<()> {
    println!();
    println!(
        "  {}",
        crate::theme::style_bold_header("\u{1f4dd} Prompt Library", theme),
    );
    println!(
        "  {}",
        crate::theme::style_border(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            theme
        )
    );
    println!(
        "  {} Coming in a future release.",
        crate::theme::style_muted("\u{23f3}", theme),
    );
    println!(
        "  {} Will support TOML-based prompt CRUD at {}.",
        crate::theme::style_muted("\u{2139}", theme),
        crate::theme::style_value("~/.forge/prompts/", theme),
    );
    println!();
    Ok(())
}
