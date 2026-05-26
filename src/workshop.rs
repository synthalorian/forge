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

/// Helper: run a command and return trimmed stdout, or error message.
fn cmd_stdout(program: &str, args: &[&str], dir: &std::path::Path) -> Result<String> {
    let out = std::process::Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute `{}`: {}", program, e))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        anyhow::bail!("`{}` failed: {}", program, stderr.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Helper: run a command, check success, return Ok(status.success).
fn cmd_status(program: &str, args: &[&str], dir: &std::path::Path) -> Result<bool> {
    let status = std::process::Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to execute `{}`: {}", program, e))?;
    Ok(status.success())
}

/// `forge cast` — Create a GitHub release with binary and asset uploads.
pub fn run_cast(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    println!();
    println!(
        "{}",
        crate::theme::style_bold_header("🚀 Forge Cast — GitHub Release", theme)
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
    println!();

    // ── Step 1: Detect project ──────────────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Step 1: Detecting project", theme)
    );

    // Must be a git repo
    let repo_root = cmd_stdout("git", &["rev-parse", "--show-toplevel"], &cwd)
        .map_err(|_| anyhow::anyhow!("Not a git repository — run `forge cast` from inside a git repo"))?;
    let repo_root = std::path::Path::new(&repo_root).to_path_buf();

    println!(
        "  {} Git root: {}",
        crate::theme::style_success("✓", theme),
        crate::theme::style_muted(&repo_root.display().to_string(), theme),
    );

    // Get remote origin to derive owner/repo
    let remote_url = cmd_stdout(
        "git",
        &["remote", "get-url", "origin"],
        &repo_root,
    )
    .map_err(|_| anyhow::anyhow!("No git remote 'origin' configured — cannot determine GitHub repo"))?;

    // Parse owner/repo from remote URL (supports git@github.com:owner/repo and https://github.com/owner/repo)
    let repo_slug = remote_url
        .trim()
        .trim_start_matches("https://github.com/")
        .trim_start_matches("git@github.com:")
        .trim_end_matches(".git");
    let repo_slug = repo_slug.to_string();

    if !repo_slug.contains('/') {
        anyhow::bail!("Could not parse GitHub owner/repo from remote URL: {}", remote_url);
    }

    println!(
        "  {} GitHub repo: {}",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value(&repo_slug, theme),
    );

    // Get version from Cargo.toml
    let cargo_toml = repo_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found — forge cast currently supports Rust projects");
    }

    let cargo_content = std::fs::read_to_string(&cargo_toml)?;
    let version_line = cargo_content
        .lines()
        .find(|l| l.trim().starts_with("version = "))
        .ok_or_else(|| anyhow::anyhow!("Could not find version in Cargo.toml"))?;
    let version = version_line
        .trim()
        .trim_start_matches("version = ")
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    println!(
        "  {} Version: {}",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value(&version, theme),
    );

    // Check branch — we want main or master
    let branch = cmd_stdout("git", &["branch", "--show-current"], &repo_root).unwrap_or_default();
    if !branch.is_empty() {
        println!(
            "  {} Branch: {}",
            crate::theme::style_label("Branch:", theme),
            crate::theme::style_accent(&branch, theme),
        );
    }

    // Check for uncommitted changes
    let has_uncommitted = !cmd_status("git", &["diff", "--quiet"], &repo_root).unwrap_or(true);
    if has_uncommitted {
        println!(
            "  {} Uncommitted changes detected",
            crate::theme::style_error("⚠", theme),
        );
    } else {
        println!(
            "  {} Working tree clean",
            crate::theme::style_success("✓", theme),
        );
    }

    println!();

    // ── Step 2: Build release binary ────────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Step 2: Building release binary", theme)
    );

    let build_ok = cmd_status("cargo", &["build", "--release"], &repo_root)?;
    if !build_ok {
        anyhow::bail!("cargo build --release failed");
    }
    println!(
        "  {} Release binary built",
        crate::theme::style_success("✓", theme),
    );
    println!();

    // ── Step 3: Create GitHub release ───────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Step 3: Creating GitHub release", theme)
    );

    // Check gh CLI availability
    let gh_available = cmd_status("which", &["gh"], &cwd).unwrap_or(false)
        || cmd_status("command", &["-v", "gh"], &cwd).unwrap_or(false);
    if !gh_available {
        anyhow::bail!("GitHub CLI (`gh`) is not installed. Install it from https://cli.github.com/");
    }

    // Check gh auth status
    let auth_ok = cmd_status("gh", &["auth", "status"], &cwd).unwrap_or(false);
    if !auth_ok {
        anyhow::bail!("Not authenticated with GitHub CLI. Run `gh auth login` first.");
    }
    println!(
        "  {} GitHub CLI authenticated",
        crate::theme::style_success("✓", theme),
    );

    // Generate release notes from git log since last tag
    let last_tag = cmd_stdout("git", &["describe", "--tags", "--abbrev=0"], &repo_root).ok();
    let release_notes = if let Some(ref tag) = last_tag {
        cmd_stdout(
            "git",
            &["log", "--oneline", "--no-decorate", &format!("{}..HEAD", tag)],
            &repo_root,
        )
        .unwrap_or_default()
    } else {
        // First release — get all commits
        cmd_stdout("git", &["log", "--oneline", "--no-decorate"], &repo_root).unwrap_or_default()
    };

    let tag_name = format!("v{}", version);
    let title = format!("v{}", version);

    // Write notes to temp file for gh release create
    let notes_content = if release_notes.is_empty() {
        format!("Release version {}", version)
    } else {
        format!("Release version {}\n\n{}", version, release_notes)
    };

    let notes_path = repo_root.join(".forge_release_notes.md");
    std::fs::write(&notes_path, &notes_content)?;

    // Build the gh release create command
    let binary_path = repo_root.join("target/release/forge");
    let binary_str = binary_path.to_string_lossy().to_string();

    println!(
        "  {} Creating release {}...",
        crate::theme::style_label("→", theme),
        crate::theme::style_value(&tag_name, theme),
    );

    // gh release create v<VERSION> <binary> --title <title> --notes-file <file> --repo <slug>
    let create_result = std::process::Command::new("gh")
        .args([
            "release",
            "create",
            &tag_name,
            &binary_str,
            "--title",
            &title,
            "--notes-file",
            &notes_path.to_string_lossy(),
            "--repo",
            &repo_slug,
        ])
        .current_dir(&repo_root)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run `gh release create`: {}", e))?;

    if !create_result.success() {
        // Clean up temp notes file
        let _ = std::fs::remove_file(&notes_path);
        anyhow::bail!("GitHub release creation failed");
    }

    // Clean up temp notes file
    let _ = std::fs::remove_file(&notes_path);

    println!(
        "  {} Release {} created",
        crate::theme::style_success("✓", theme),
        crate::theme::style_value(&tag_name, theme),
    );
    println!();

    // ── Step 4: Upload assets ───────────────────────────────────
    println!(
        "  {}",
        crate::theme::style_header("Step 4: Uploading assets", theme)
    );

    let mut assets_uploaded = 0u64;

    // Upload forge-hub.tar.gz if hub/ exists
    let hub_dir = repo_root.join("hub");
    if hub_dir.is_dir() {
        let tarball_path = repo_root.join("forge-hub.tar.gz");
        println!(
            "  {} Creating hub tarball...",
            crate::theme::style_label("→", theme),
        );

        // Create tarball from hub/ directory using tar
        let tar_ok = cmd_status(
            "tar",
            &["czf", &tarball_path.to_string_lossy(), "-C", &repo_root.to_string_lossy(), "hub"],
            &repo_root,
        ).unwrap_or(false);

        if tar_ok {
            println!(
                "  {} Hub tarball created",
                crate::theme::style_success("✓", theme),
            );

            // Upload tarball to release
            let upload_ok = cmd_status(
                "gh",
                &[
                    "release",
                    "upload",
                    &tag_name,
                    &tarball_path.to_string_lossy(),
                    "--repo",
                    &repo_slug,
                    "--clobber",
                ],
                &repo_root,
            ).unwrap_or(false);

            if upload_ok {
                println!(
                    "  {} forge-hub.tar.gz uploaded",
                    crate::theme::style_success("✓", theme),
                );
                assets_uploaded += 1;
            } else {
                println!(
                    "  {} Failed to upload forge-hub.tar.gz",
                    crate::theme::style_error("✗", theme),
                );
            }

            // Clean up local tarball
            let _ = std::fs::remove_file(&tarball_path);
        } else {
            println!(
                "  {} Failed to create hub tarball",
                crate::theme::style_error("✗", theme),
            );
        }
    } else {
        println!(
            "  {} No hub/ directory found — skipping hub tarball",
            crate::theme::style_muted("ℹ", theme),
        );
    }

    // Upload assets/forge-icon.png if it exists
    let icon_path = repo_root.join("assets/forge-icon.png");
    if icon_path.exists() {
        println!(
            "  {} Uploading forge-icon.png...",
            crate::theme::style_label("→", theme),
        );

        let upload_ok = cmd_status(
            "gh",
            &[
                "release",
                "upload",
                &tag_name,
                &icon_path.to_string_lossy(),
                "--repo",
                &repo_slug,
                "--clobber",
            ],
            &repo_root,
        ).unwrap_or(false);

        if upload_ok {
            println!(
                "  {} forge-icon.png uploaded",
                crate::theme::style_success("✓", theme),
            );
            assets_uploaded += 1;
        } else {
            println!(
                "  {} Failed to upload forge-icon.png",
                crate::theme::style_error("✗", theme),
            );
        }
    } else {
        println!(
            "  {} No assets/forge-icon.png found — skipping icon upload",
            crate::theme::style_muted("ℹ", theme),
        );
    }

    println!();

    // ── Summary ─────────────────────────────────────────────────
    if assets_uploaded > 0 {
        println!(
            "  {} {} asset(s) uploaded. Cast complete! 🎉",
            crate::theme::style_success("✓", theme),
            crate::theme::style_value(&assets_uploaded.to_string(), theme),
        );
    } else {
        println!(
            "  {} Release created with binary. No additional assets.",
            crate::theme::style_success("✓", theme),
        );
    }
    println!(
        "  {} https://github.com/{}/releases/tag/{}",
        crate::theme::style_label("Release URL:", theme),
        crate::theme::style_value(&repo_slug, theme),
        crate::theme::style_value(&tag_name, theme),
    );
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
