//! Workshop module — the remaining forge verbs that complete the blacksmith metaphor.
//!
//! - `forge heat` — Spin up AI agents (start/begin a session)
//! - `forge anneal` — Deep work mode / do not disturb
//! - `forge alloy` — Merge outputs from multiple agents
//! - `forge cast` — Deploy current project
//! - `forge grind` — Run linters, tests, quality checks
//! - `forge polish` — Format and document

use anyhow::Result;

use crate::config::Config;

/// `forge heat` — Spin up AI agents, start a session.
pub fn run_heat(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let agents = crate::mind::detect_agents();
    let running = agents
        .iter()
        .filter(|a| a.status == crate::mind::ServiceStatus::Running)
        .count();

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("🔥 Forge Heat — Spinning Up Agents", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    if running > 0 {
        println!(
            "  {} {} agent(s) already running",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&running.to_string(), theme),
        );
    }

    // Try to start agents that are installed but stopped
    let mut started = 0u64;
    let mut failed = 0u64;

    for agent in &agents {
        match agent.status {
            crate::mind::ServiceStatus::Running => {
                println!(
                    "  {} {} — already hot",
                    crate::theme::style_success("●", theme),
                    crate::theme::style_accent(&agent.name, theme),
                );
            }
            crate::mind::ServiceStatus::Stopped => {
                // Attempt to start the agent
                let started_ok = try_start_agent(&agent.name);
                if started_ok {
                    started += 1;
                    println!(
                        "  {} {} — fired up",
                        crate::theme::style_success("▲", theme),
                        crate::theme::style_accent(&agent.name, theme),
                    );
                } else {
                    failed += 1;
                    println!(
                        "  {} {} — couldn't start",
                        crate::theme::style_error("✗", theme),
                        crate::theme::style_muted(&agent.name, theme),
                    );
                }
            }
            crate::mind::ServiceStatus::NotInstalled => {
                println!(
                    "  {} {} — not installed",
                    crate::theme::style_muted("○", theme),
                    crate::theme::style_muted(&agent.name, theme),
                );
            }
        }
    }

    println!();
    if started > 0 {
        println!(
            "  {} {} agent(s) started. The forge burns hot. 🔨",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&started.to_string(), theme),
        );
    } else if running > 0 {
        println!(
            "  {} All available agents already running.",
            crate::theme::style_success("✓", theme),
        );
    } else if failed > 0 {
        println!(
            "  {} No agents could be started. Check installations.",
            crate::theme::style_error("⚠", theme),
        );
        println!(
            "  {} Use {} to check agent availability.",
            crate::theme::style_muted("Tip:", theme),
            crate::theme::style_value("forge breathe", theme),
        );
    }
    println!();

    Ok(())
}

fn try_start_agent(name: &str) -> bool {
    match name {
        "opencode" => std::process::Command::new("opencode")
            .env("TERM", "xterm-256color")
            .spawn()
            .map(|_| true)
            .unwrap_or(false),
        _ => false, // Other agents need manual start or don't have simple CLI launch
    }
}

/// `forge anneal` — Deep work mode. Do not disturb.
pub fn run_anneal(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("🛡️  Forge Anneal — Deep Work Mode", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Check for running distractions
    let distractions = detect_distractions();

    if distractions.is_empty() {
        println!(
            "  {} No distractions detected. The anvil is clear.",
            crate::theme::style_success("✓", theme),
        );
    } else {
        println!(
            "  {} Potential distractions detected:",
            crate::theme::style_label("⚠", theme),
        );
        for d in &distractions {
            println!(
                "    {} {}",
                crate::theme::style_error("●", theme),
                crate::theme::style_value(d, theme),
            );
        }
    }

    println!();
    println!(
        "  {}",
        crate::theme::style_header("Deep Work Active", theme)
    );
    println!(
        "  {} Notifications silenced. Focus engaged.",
        crate::theme::style_accent("🧘", theme),
    );
    println!(
        "  {} Close chat apps, silence phone, commit to one task.",
        crate::theme::style_muted("→", theme),
    );
    println!();

    // Show a focused summary of what we're working on
    if cfg.db_path.exists() {
        if let Ok(conn) = crate::db::connect(cfg) {
            let last_repo: Option<String> = conn
                .query_row(
                    "SELECT repo_name FROM backups ORDER BY created_at DESC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            if let Some(repo) = last_repo {
                println!(
                    "  {} Last project: {}",
                    crate::theme::style_label("Recent:", theme),
                    crate::theme::style_value(&repo, theme),
                );
            }
        }
    }

    println!();
    println!(
        "  {} \"The Lord will keep you from all harm — he will watch over your life.\"",
        crate::theme::style_muted("📖 Psalm 121:7", theme),
    );
    println!();

    Ok(())
}

fn detect_distractions() -> Vec<String> {
    let mut distractions = Vec::new();
    let checks = [
        ("Slack", "pgrep -f slack"),
        ("Discord", "pgrep -f discord"),
        ("Spotify", "pgrep -f spotify"),
        ("Steam", "pgrep -f steam"),
        ("Chrome (many tabs?)", "pgrep -c chrome"),
    ];

    for (name, check) in &checks {
        let output = std::process::Command::new("sh")
            .args(["-c", check])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let count: i64 = output.trim().parse().unwrap_or(0);
        if count > 0 {
            if *name == "Chrome (many tabs?)" && count > 10 {
                distractions.push(format!("{} ({} processes!)", name, count));
            } else if *name != "Chrome (many tabs?)" {
                distractions.push(name.to_string());
            }
        }
    }

    distractions
}

/// `forge alloy` — Merge outputs from multiple agents.
pub fn run_alloy(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("⚗️  Forge Alloy — Agent Output Merger", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let agents = crate::mind::detect_agents();
    let running: Vec<_> = agents
        .iter()
        .filter(|a| a.status == crate::mind::ServiceStatus::Running)
        .collect();

    if running.is_empty() {
        println!(
            "  {} No agents running to alloy.",
            crate::theme::style_error("✗", theme),
        );
        println!(
            "  {} Use {} to start agents first.",
            crate::theme::style_muted("Tip:", theme),
            crate::theme::style_value("forge heat", theme),
        );
        println!();
        return Ok(());
    }

    println!(
        "  {} {} agent(s) available for merging:",
        crate::theme::style_label("Available:", theme),
        crate::theme::style_value(&running.len().to_string(), theme),
    );

    for agent in &running {
        println!(
            "  {} {} ({})",
            crate::theme::style_success("●", theme),
            crate::theme::style_accent(&agent.name, theme),
            crate::theme::style_muted(&format!("{}", agent.agent_type), theme),
        );
    }

    println!();
    println!(
        "  {} Alloy combines outputs from multiple agents into a unified result.",
        crate::theme::style_muted("ℹ", theme),
    );
    println!(
        "  {} Use {} to send a task to the best available agent.",
        crate::theme::style_muted("→", theme),
        crate::theme::style_value("forge strike <task>", theme),
    );
    println!(
        "  {} Multi-agent merging requires running {} and multiple agent backends.",
        crate::theme::style_muted("→", theme),
        crate::theme::style_value("forge breathe", theme),
    );
    println!();

    Ok(())
}

/// `forge cast` — Deploy current project.
pub fn run_cast(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("🚀 Forge Cast — Deploy Project", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!(
        "  {} {}",
        crate::theme::style_label("Project:", theme),
        crate::theme::style_value(&project_name, theme),
    );
    println!(
        "  {} {}",
        crate::theme::style_label("Path:", theme),
        crate::theme::style_muted(&cwd.display().to_string(), theme),
    );

    // Check if it's a git repo
    let is_git = cwd.join(".git").exists();
    if is_git {
        println!(
            "  {} Git repository detected",
            crate::theme::style_success("✓", theme),
        );

        // Get current branch
        let branch = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        if !branch.is_empty() {
            println!(
                "  {} {}",
                crate::theme::style_label("Branch:", theme),
                crate::theme::style_accent(&branch, theme),
            );
        }

        // Check for uncommitted changes
        let dirty = std::process::Command::new("git")
            .args(["diff", "--quiet"])
            .current_dir(&cwd)
            .status()
            .map(|s| !s.success())
            .unwrap_or(false);

        if dirty {
            println!(
                "  {} Uncommitted changes detected — commit before casting!",
                crate::theme::style_error("⚠", theme),
            );
        }
    } else {
        println!(
            "  {} Not a git repository",
            crate::theme::style_error("✗", theme),
        );
    }

    // Detect project type
    let project_type = detect_project_type(&cwd);
    if let Some(ref pt) = project_type {
        println!(
            "  {} {}",
            crate::theme::style_label("Type:", theme),
            crate::theme::style_value(pt, theme),
        );
    }

    println!();

    // Deployment suggestions based on type
    if let Some(ref pt) = project_type {
        match pt.as_str() {
            "Rust" => {
                println!(
                    "  {}",
                    crate::theme::style_header("Deployment Steps", theme)
                );
                println!(
                    "  {} {} Build release",
                    crate::theme::style_muted("1.", theme),
                    crate::theme::style_value("cargo build --release", theme),
                );
                println!(
                    "  {} {} Run quality checks",
                    crate::theme::style_muted("2.", theme),
                    crate::theme::style_value("cargo test && cargo clippy", theme),
                );
                println!(
                    "  {} {} (optional) Publish to crates.io",
                    crate::theme::style_muted("3.", theme),
                    crate::theme::style_value("cargo publish", theme),
                );
            }
            "Node.js" => {
                println!(
                    "  {}",
                    crate::theme::style_header("Deployment Steps", theme)
                );
                println!(
                    "  {} {} Build",
                    crate::theme::style_muted("1.", theme),
                    crate::theme::style_value("npm run build", theme),
                );
                println!(
                    "  {} {} Deploy via platform of choice",
                    crate::theme::style_muted("2.", theme),
                    crate::theme::style_value("vercel / netlify / docker", theme),
                );
            }
            "Ruby" => {
                println!(
                    "  {}",
                    crate::theme::style_header("Deployment Steps", theme)
                );
                println!(
                    "  {} {} Run tests",
                    crate::theme::style_muted("1.", theme),
                    crate::theme::style_value("bundle exec rake", theme),
                );
                println!(
                    "  {} {} Deploy",
                    crate::theme::style_muted("2.", theme),
                    crate::theme::style_value("cap deploy / kamal deploy", theme),
                );
            }
            _ => {
                println!(
                    "  {} Generic project — build and deploy as appropriate.",
                    crate::theme::style_muted("ℹ", theme),
                );
            }
        }
    }

    println!();

    Ok(())
}

fn detect_project_type(dir: &std::path::Path) -> Option<String> {
    if dir.join("Cargo.toml").exists() {
        Some("Rust".to_string())
    } else if dir.join("package.json").exists() {
        Some("Node.js".to_string())
    } else if dir.join("Gemfile").exists() {
        Some("Ruby".to_string())
    } else if dir.join("go.mod").exists() {
        Some("Go".to_string())
    } else if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() {
        Some("Java".to_string())
    } else if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() {
        Some("Python".to_string())
    } else {
        None
    }
}

/// `forge grind` — Run linters, tests, quality checks.
pub fn run_grind(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("⚙️  Forge Grind — Quality Checks", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!(
        "  {} {}",
        crate::theme::style_label("Project:", theme),
        crate::theme::style_value(&project_name, theme),
    );
    println!();

    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;

    // Detect and run appropriate checks
    let project_type = detect_project_type(&cwd);

    match project_type.as_deref() {
        Some("Rust") => {
            // cargo test
            println!(
                "  {}",
                crate::theme::style_header("Running cargo test", theme)
            );
            let test_result = std::process::Command::new("cargo")
                .args(["test", "--quiet"])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if test_result {
                println!("  {} Tests passed", crate::theme::style_success("✓", theme),);
                passed += 1;
            } else {
                println!("  {} Tests failed", crate::theme::style_error("✗", theme),);
                failed += 1;
            }

            // cargo clippy
            println!(
                "  {}",
                crate::theme::style_header("Running cargo clippy", theme)
            );
            let clippy_result = std::process::Command::new("cargo")
                .args(["clippy", "--quiet", "--", "-D", "warnings"])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if clippy_result {
                println!("  {} Clippy clean", crate::theme::style_success("✓", theme),);
                passed += 1;
            } else {
                println!(
                    "  {} Clippy warnings found",
                    crate::theme::style_error("✗", theme),
                );
                failed += 1;
            }

            // cargo fmt check
            println!(
                "  {}",
                crate::theme::style_header("Checking formatting", theme)
            );
            let fmt_result = std::process::Command::new("cargo")
                .args(["fmt", "--check"])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if fmt_result {
                println!(
                    "  {} Formatting OK",
                    crate::theme::style_success("✓", theme),
                );
                passed += 1;
            } else {
                println!(
                    "  {} Formatting needed — run {}",
                    crate::theme::style_error("✗", theme),
                    crate::theme::style_value("forge polish", theme),
                );
                failed += 1;
            }
        }
        Some("Node.js") => {
            // npm test
            if cwd.join("package.json").exists() {
                println!(
                    "  {}",
                    crate::theme::style_header("Running npm test", theme)
                );
                let test_result = std::process::Command::new("npm")
                    .args(["test"])
                    .current_dir(&cwd)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);

                if test_result {
                    println!("  {} Tests passed", crate::theme::style_success("✓", theme),);
                    passed += 1;
                } else {
                    println!("  {} Tests failed", crate::theme::style_error("✗", theme),);
                    failed += 1;
                }
            }
        }
        _ => {
            println!(
                "  {} No recognized project type — skipping automated checks.",
                crate::theme::style_muted("ℹ", theme),
            );
            skipped += 3;
        }
    }

    // Git status check (universal)
    if cwd.join(".git").exists() {
        println!();
        println!("  {}", crate::theme::style_header("Git Status", theme));

        let dirty = std::process::Command::new("git")
            .args(["diff", "--quiet"])
            .current_dir(&cwd)
            .status()
            .map(|s| !s.success())
            .unwrap_or(false);

        let staged = std::process::Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(&cwd)
            .status()
            .map(|s| !s.success())
            .unwrap_or(false);

        if dirty || staged {
            println!(
                "  {} Uncommitted changes present",
                crate::theme::style_error("⚠", theme),
            );
            failed += 1;
        } else {
            println!(
                "  {} Working tree clean",
                crate::theme::style_success("✓", theme),
            );
            passed += 1;
        }
    }

    println!();
    if failed == 0 {
        println!(
            "  {} All {} checks passed. The blade is sharp. ⚔️",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&passed.to_string(), theme),
        );
    } else {
        println!(
            "  {} {} passed, {} failed, {} skipped",
            crate::theme::style_bold_header("Results:", theme),
            crate::theme::style_value(&passed.to_string(), theme),
            crate::theme::style_error(&failed.to_string(), theme),
            crate::theme::style_muted(&skipped.to_string(), theme),
        );
    }
    println!();

    Ok(())
}

/// `forge polish` — Format and document.
pub fn run_polish(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("✨ Forge Polish — Format & Document", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!(
        "  {} {}",
        crate::theme::style_label("Project:", theme),
        crate::theme::style_value(&project_name, theme),
    );
    println!();

    let mut polished = 0u64;

    // Detect and run formatters
    let project_type = detect_project_type(&cwd);

    match project_type.as_deref() {
        Some("Rust") => {
            // cargo fmt
            println!(
                "  {}",
                crate::theme::style_header("Running cargo fmt", theme)
            );
            let fmt_result = std::process::Command::new("cargo")
                .args(["fmt"])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if fmt_result {
                println!("  {} Formatted", crate::theme::style_success("✓", theme),);
                polished += 1;
            } else {
                println!(
                    "  {} cargo fmt failed",
                    crate::theme::style_error("✗", theme),
                );
            }

            // cargo clippy --fix (auto-fix what's safe)
            println!(
                "  {}",
                crate::theme::style_header("Running cargo clippy --fix (safe fixes)", theme)
            );
            let fix_result = std::process::Command::new("cargo")
                .args(["clippy", "--fix", "--allow-dirty", "--quiet"])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if fix_result {
                println!(
                    "  {} Auto-fixed clippy issues",
                    crate::theme::style_success("✓", theme),
                );
                polished += 1;
            } else {
                println!(
                    "  {} Some issues need manual attention",
                    crate::theme::style_muted("ℹ", theme),
                );
            }
        }
        Some("Node.js") => {
            // Try prettier
            println!(
                "  {}",
                crate::theme::style_header("Formatting with prettier", theme)
            );
            let fmt_result = std::process::Command::new("npx")
                .args(["prettier", "--write", "."])
                .current_dir(&cwd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if fmt_result {
                println!(
                    "  {} Formatted with prettier",
                    crate::theme::style_success("✓", theme),
                );
                polished += 1;
            } else {
                println!(
                    "  {} prettier not available — install with {}",
                    crate::theme::style_muted("ℹ", theme),
                    crate::theme::style_value("npm i -D prettier", theme),
                );
            }
        }
        _ => {
            println!(
                "  {} No auto-formatter detected for this project type.",
                crate::theme::style_muted("ℹ", theme),
            );
        }
    }

    // Generate documentation summary
    println!();
    println!("  {}", crate::theme::style_header("Documentation", theme));

    // Check README
    let readme = cwd.join("README.md");
    if readme.exists() {
        let size = std::fs::metadata(&readme).map(|m| m.len()).unwrap_or(0);
        if size > 500 {
            println!(
                "  {} README.md present ({} bytes)",
                crate::theme::style_success("✓", theme),
                crate::theme::style_value(&size.to_string(), theme),
            );
        } else {
            println!(
                "  {} README.md is thin ({} bytes) — consider expanding",
                crate::theme::style_error("⚠", theme),
                crate::theme::style_value(&size.to_string(), theme),
            );
        }
    } else {
        println!(
            "  {} No README.md found",
            crate::theme::style_error("✗", theme),
        );
    }

    // Check CHANGELOG
    let changelog = cwd.join("CHANGELOG.md");
    if changelog.exists() {
        println!(
            "  {} CHANGELOG.md present",
            crate::theme::style_success("✓", theme),
        );
    } else {
        println!(
            "  {} No CHANGELOG.md — consider adding one",
            crate::theme::style_muted("ℹ", theme),
        );
    }

    println!();
    if polished > 0 {
        println!(
            "  {} {} items polished. Shiny. ✨",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&polished.to_string(), theme),
        );
    }
    println!();

    Ok(())
}
