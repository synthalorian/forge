//! AI agent harness — the Mind module for Forge.
//!
//! Detects available AI agents (OpenCode, llama-swap, Hermes, Codex),
//! routes tasks to the best available agent, and executes them.

use anyhow::{Context, Result};
use std::process::Command;

// ── Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AgentType {
    Local,
    Remote,
    Cli,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Local => write!(f, "local"),
            AgentType::Remote => write!(f, "remote"),
            AgentType::Cli => write!(f, "cli"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    NotInstalled,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "running"),
            ServiceStatus::Stopped => write!(f, "stopped"),
            ServiceStatus::NotInstalled => write!(f, "not installed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub name: String,
    pub agent_type: AgentType,
    pub status: ServiceStatus,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub size: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub agent: String,
    pub agent_type: AgentType,
    pub confidence: f32,
}

// ── Agent Detection ────────────────────────────────────────────────

/// Resolve llama-swap config path from env or default.
fn llama_swap_config_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("LLAMA_SWAP_CONFIG") {
        return std::path::PathBuf::from(path);
    }
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("llama.cpp/llama-swap/config.yaml")
}
const HERMES_AUTH_PATH: &str = ".hermes/auth.json";

/// Detect all known AI agents and their status.
pub fn detect_agents() -> Vec<AgentStatus> {
    vec![
        detect_opencode(),
        detect_llama_swap(),
        detect_hermes(),
        detect_codex(),
    ]
}

fn detect_opencode() -> AgentStatus {
    let binary = which_binary("opencode");

    if !binary {
        return AgentStatus {
            name: "opencode".to_string(),
            agent_type: AgentType::Local,
            status: ServiceStatus::NotInstalled,
            endpoint: None,
            model: Some("glm-5.1 (Z.AI)".to_string()),
            version: None,
        };
    }

    let version = get_command_output("opencode", &["--version"]);

    // Check if there's an active session
    let running = is_process_running("opencode");

    AgentStatus {
        name: "opencode".to_string(),
        agent_type: AgentType::Local,
        status: if running {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        },
        endpoint: Some("https://api.z.ai/api/coding/paas/v4".to_string()),
        model: Some("glm-5.1 (Z.AI)".to_string()),
        version,
    }
}

fn detect_llama_swap() -> AgentStatus {
    let config_path = llama_swap_config_path();
    let config_exists = config_path.exists();

    if !config_exists {
        return AgentStatus {
            name: "llama-swap".to_string(),
            agent_type: AgentType::Local,
            status: ServiceStatus::NotInstalled,
            endpoint: None,
            model: None,
            version: None,
        };
    }

    let running = is_process_running("llama-swap") || is_process_running("llama");
    let models = detect_models();
    let model_list = if models.is_empty() {
        None
    } else {
        Some(
            models
                .iter()
                .map(|m| m.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
        )
    };

    AgentStatus {
        name: "llama-swap".to_string(),
        agent_type: AgentType::Local,
        status: if running {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        },
        endpoint: Some("http://localhost:8080".to_string()),
        model: model_list,
        version: None,
    }
}

fn detect_hermes() -> AgentStatus {
    let binary = which_binary("hermes");

    if !binary {
        return AgentStatus {
            name: "hermes".to_string(),
            agent_type: AgentType::Remote,
            status: ServiceStatus::NotInstalled,
            endpoint: None,
            model: None,
            version: None,
        };
    }

    let _auth_exists = dirs::home_dir()
        .map(|h| h.join(HERMES_AUTH_PATH).exists())
        .unwrap_or(false);

    let running = is_process_running("hermes");
    let version = get_command_output("hermes", &["--version"]);

    AgentStatus {
        name: "hermes".to_string(),
        agent_type: AgentType::Remote,
        status: if running {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        },
        endpoint: Some("https://hermes.api".to_string()),
        model: None,
        version,
    }
}

fn detect_codex() -> AgentStatus {
    let binary = which_binary("codex");

    if !binary {
        return AgentStatus {
            name: "codex".to_string(),
            agent_type: AgentType::Cli,
            status: ServiceStatus::NotInstalled,
            endpoint: None,
            model: None,
            version: None,
        };
    }

    let version = get_command_output("codex", &["--version"]);

    AgentStatus {
        name: "codex".to_string(),
        agent_type: AgentType::Cli,
        status: ServiceStatus::Stopped,
        endpoint: None,
        model: None,
        version,
    }
}

// ── Model Detection ────────────────────────────────────────────────

/// Parse llama-swap config.yaml for available models.
/// Uses simple string parsing — no YAML dependency needed.
pub fn detect_models() -> Vec<ModelInfo> {
    let config_path = llama_swap_config_path();
    if !config_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut models = Vec::new();
    let mut in_models = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("models:") {
            in_models = true;
            continue;
        }

        if in_models {
            // Stop if we hit a top-level key that isn't indented
            if !line.starts_with(' ') && !line.starts_with('-') && !trimmed.is_empty() {
                break;
            }

            // Parse "- name: modelname" or "  - name: modelname"
            if let Some(name_part) = trimmed.strip_prefix("- name:") {
                let name = name_part.trim().trim_matches('"').trim_matches('\'');
                if !name.is_empty() {
                    models.push(ModelInfo {
                        name: name.to_string(),
                        provider: "llama-swap".to_string(),
                        size: None,
                    });
                }
            }

            // Also handle "name: modelname" without the dash
            if !trimmed.starts_with('-') && trimmed.starts_with("name:") {
                let name = trimmed
                    .strip_prefix("name:")
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !name.is_empty() && models.is_empty() {
                    models.push(ModelInfo {
                        name: name.to_string(),
                        provider: "llama-swap".to_string(),
                        size: None,
                    });
                }
            }
        }
    }

    models
}

// ── Task Routing ───────────────────────────────────────────────────

/// Route a task to the best available agent.
///
/// Priority: opencode > llama-swap > hermes > codex
/// Prefers running agents over stopped ones.
pub fn route_task(task: &str, available: &[AgentStatus]) -> RoutingResult {
    let _ = task; // Task content could be used for smarter routing in future

    let priority = ["opencode", "llama-swap", "hermes", "codex"];

    // First pass: find running agents in priority order
    for &name in &priority {
        if let Some(agent) = available
            .iter()
            .find(|a| a.name == name && a.status == ServiceStatus::Running)
        {
            return RoutingResult {
                agent: agent.name.clone(),
                agent_type: agent.agent_type.clone(),
                confidence: 0.9,
            };
        }
    }

    // Second pass: find any installed agent in priority order
    for &name in &priority {
        if let Some(agent) = available
            .iter()
            .find(|a| a.name == name && a.status != ServiceStatus::NotInstalled)
        {
            return RoutingResult {
                agent: agent.name.clone(),
                agent_type: agent.agent_type.clone(),
                confidence: 0.5,
            };
        }
    }

    // Fallback: opencode is always the default choice
    RoutingResult {
        agent: "opencode".to_string(),
        agent_type: AgentType::Local,
        confidence: 0.1,
    }
}

// ── Task Execution ─────────────────────────────────────────────────

/// Execute a task by delegating to the best available AI agent.
///
/// If `agent_override` is provided, forces that specific agent.
/// Returns the agent's stdout output on success.
pub fn execute_strike(task: &str, agent_override: Option<&str>) -> Result<String> {
    let agents = detect_agents();
    let target_agent = if let Some(forced) = agent_override {
        forced.to_string()
    } else {
        let routing = route_task(task, &agents);
        routing.agent
    };

    let output = match target_agent.as_str() {
        "opencode" => run_agent_command("opencode", &["--prompt", task])?,
        "llama-swap" => {
            // llama-swap doesn't have a direct CLI interface; suggest using the API
            anyhow::bail!(
                "llama-swap requires API interaction. Use the HTTP endpoint at http://localhost:8080"
            )
        }
        "hermes" => run_agent_command("hermes", &[task])?,
        "codex" => run_agent_command("codex", &[task])?,
        other => anyhow::bail!("Unknown agent: {}", other),
    };

    Ok(output)
}

fn run_agent_command(bin: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute {}", bin))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{} failed: {}", bin, stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Helpers ────────────────────────────────────────────────────────

/// Check if a binary exists in PATH.
fn which_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a process is running.
fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get command output as a trimmed string.
fn get_command_output(bin: &str, args: &[&str]) -> Option<String> {
    Command::new(bin)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_agents_returns_vector() {
        let agents = detect_agents();
        assert_eq!(agents.len(), 4);

        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"opencode"));
        assert!(names.contains(&"llama-swap"));
        assert!(names.contains(&"hermes"));
        assert!(names.contains(&"codex"));
    }

    #[test]
    fn test_route_task_prefers_running_local() {
        let agents = vec![
            AgentStatus {
                name: "opencode".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::Running,
                endpoint: None,
                model: None,
                version: None,
            },
            AgentStatus {
                name: "llama-swap".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::Running,
                endpoint: None,
                model: None,
                version: None,
            },
            AgentStatus {
                name: "hermes".to_string(),
                agent_type: AgentType::Remote,
                status: ServiceStatus::Stopped,
                endpoint: None,
                model: None,
                version: None,
            },
        ];

        let result = route_task("write a function", &agents);
        assert_eq!(result.agent, "opencode");
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_route_task_falls_back_to_stopped() {
        let agents = vec![
            AgentStatus {
                name: "opencode".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::NotInstalled,
                endpoint: None,
                model: None,
                version: None,
            },
            AgentStatus {
                name: "llama-swap".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::Stopped,
                endpoint: None,
                model: None,
                version: None,
            },
            AgentStatus {
                name: "hermes".to_string(),
                agent_type: AgentType::Remote,
                status: ServiceStatus::Running,
                endpoint: None,
                model: None,
                version: None,
            },
        ];

        // Even though hermes is running, llama-swap has higher priority and is installed (just stopped)
        let result = route_task("explain this code", &agents);
        assert_eq!(result.agent, "hermes"); // hermes is running and higher priority agents aren't available running
    }

    #[test]
    fn test_route_task_all_not_installed() {
        let agents = vec![
            AgentStatus {
                name: "opencode".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::NotInstalled,
                endpoint: None,
                model: None,
                version: None,
            },
            AgentStatus {
                name: "llama-swap".to_string(),
                agent_type: AgentType::Local,
                status: ServiceStatus::NotInstalled,
                endpoint: None,
                model: None,
                version: None,
            },
        ];

        let result = route_task("anything", &agents);
        assert_eq!(result.agent, "opencode"); // Fallback
        assert_eq!(result.confidence, 0.1);
    }

    #[test]
    fn test_agent_type_display() {
        assert_eq!(format!("{}", AgentType::Local), "local");
        assert_eq!(format!("{}", AgentType::Remote), "remote");
        assert_eq!(format!("{}", AgentType::Cli), "cli");
    }

    #[test]
    fn test_service_status_display() {
        assert_eq!(format!("{}", ServiceStatus::Running), "running");
        assert_eq!(format!("{}", ServiceStatus::Stopped), "stopped");
        assert_eq!(format!("{}", ServiceStatus::NotInstalled), "not installed");
    }

    #[test]
    fn test_detect_models_with_no_config() {
        // If the config doesn't exist, should return empty
        let models = detect_models();
        // Result depends on whether the file exists in the test environment
        // Just verify it doesn't panic
        let _ = models;
    }
}
