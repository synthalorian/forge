//! Anvil module — Phase 2C: Incremental backups, verification, retention, search, health.
//!
//! The Anvil is where code gets shaped. This module provides:
//! - `forge temper` — Backup verification (re-hash and compare)
//! - `forge anvil prune` — Retention policy enforcement
//! - `forge anvil search <query>` — Cross-repo code search via ripgrep
//! - `forge anvil health` — Project status dashboard (dirty, stale, missing remotes)
//! - Incremental backup support via chunk manifest comparison

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

use crate::cli::{AnvilAction, AnvilArgs};
use crate::config::Config;

// ── Incremental backup support ──────────────────────────────────────

/// Get the chunk hashes from the most recent backup of a repo.
/// Returns None if no previous backup exists.
pub fn get_last_backup_chunks(
    conn: &rusqlite::Connection,
    repo_name: &str,
) -> Result<Option<HashSet<String>>> {
    let backup_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM backups WHERE repo_name = ?1 ORDER BY created_at DESC LIMIT 1",
            [repo_name],
            |row| row.get(0),
        )
        .optional()
        .context("Failed to query last backup")?;

    match backup_id {
        Some(id) => {
            let hashes = crate::db::get_backup_chunk_hashes(conn, id)?;
            Ok(Some(hashes.into_iter().collect()))
        }
        None => Ok(None),
    }
}

/// Determine whether an incremental backup should be performed.
/// Returns true if a previous full backup exists for this repo.
pub fn can_do_incremental(conn: &rusqlite::Connection, repo_name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM backups WHERE repo_name = ?1 AND backup_type = 'full'",
        [repo_name],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

// ── forge temper (verify) ───────────────────────────────────────────

/// Verify all backups by re-hashing their archives and comparing with stored SHA-256.
pub fn run_temper(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg)?;

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Temper — Backup Verification", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    let mut stmt = conn
        .prepare(
            "SELECT id, repo_name, archive_path, sha256, backup_type, created_at FROM backups ORDER BY created_at DESC",
        )
        .context("Failed to prepare verification query")?;

    let entries: Vec<(i64, String, String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>("id")?,
                row.get::<_, String>("repo_name")?,
                row.get::<_, String>("archive_path")?,
                row.get::<_, String>("sha256")?,
                row.get::<_, String>("backup_type")?,
                row.get::<_, String>("created_at")?,
            ))
        })
        .context("Failed to query backups for verification")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse backup rows")?;

    if entries.is_empty() {
        println!(
            "{}",
            crate::theme::style_muted("No backups to verify.", theme)
        );
        return Ok(());
    }

    let mut verified = 0u64;
    let mut failed = 0u64;
    let mut missing = 0u64;

    for (id, repo_name, archive_path, expected_sha, _backup_type, created_at) in &entries {
        let manifest_path = Path::new(archive_path);

        if !manifest_path.exists() {
            println!(
                "  {} #{} {} {} — {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_value(&id.to_string(), theme),
                crate::theme::style_accent(repo_name, theme),
                crate::theme::style_muted(
                    &format!("({})", created_at.get(..10).unwrap_or(created_at)),
                    theme
                ),
                crate::theme::style_error("manifest missing", theme),
            );
            missing += 1;
            continue;
        }

        // Re-hash the manifest
        let bytes = std::fs::read(manifest_path)
            .with_context(|| format!("Failed to read manifest {}", archive_path))?;
        let actual_sha = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            hex::encode(hasher.finalize())
        };

        // Verify chunk files exist
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_content)?;
        let chunk_hashes: Vec<String> = manifest_json["chunk_hashes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let chunks_dir = cfg.archive_dir.join("chunks");
        let mut missing_chunks = 0u64;
        for hash in &chunk_hashes {
            let prefix: String = hash.chars().take(2).collect();
            let rest: String = hash.chars().skip(2).collect();
            let chunk_path = chunks_dir
                .join(&prefix)
                .join(format!("{}.zst", rest));
            if !chunk_path.exists() {
                missing_chunks += 1;
            }
        }

        let hash_ok = actual_sha.eq_ignore_ascii_case(expected_sha);
        let chunks_ok = missing_chunks == 0;

        if hash_ok && chunks_ok {
            println!(
                "  {} #{} {} {} — {}",
                crate::theme::style_success("✓", theme),
                crate::theme::style_value(&id.to_string(), theme),
                crate::theme::style_accent(repo_name, theme),
                crate::theme::style_muted(
                    &format!("({})", created_at.get(..10).unwrap_or(created_at)),
                    theme
                ),
                crate::theme::style_success("verified", theme),
            );
            verified += 1;
        } else {
            let reasons: Vec<String> = [
                (!hash_ok).then(|| "hash mismatch".to_string()),
                (!chunks_ok).then(|| format!("{} missing chunks", missing_chunks)),
            ]
            .into_iter()
            .flatten()
            .collect();
            println!(
                "  {} #{} {} {} — {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_value(&id.to_string(), theme),
                crate::theme::style_accent(repo_name, theme),
                crate::theme::style_muted(
                    &format!("({})", created_at.get(..10).unwrap_or(created_at)),
                    theme
                ),
                crate::theme::style_error(&reasons.join(", "), theme),
            );
            failed += 1;
        }
    }

    println!();
    println!(
        "  {} {} {}, {} {}, {} {}",
        crate::theme::style_bold_header("Results:", theme),
        crate::theme::style_value(&verified.to_string(), theme),
        crate::theme::style_muted("verified", theme),
        crate::theme::style_error(&failed.to_string(), theme),
        crate::theme::style_muted("failed", theme),
        crate::theme::style_value(&missing.to_string(), theme),
        crate::theme::style_muted("missing", theme),
    );

    if failed > 0 || missing > 0 {
        anyhow::bail!("Verification found issues");
    }

    Ok(())
}

// ── Retention policy enforcement ────────────────────────────────────

/// Enforce retention policy: keep_daily most recent daily backups,
/// keep_weekly weekly, keep_monthly monthly. Older backups are pruned.
pub fn run_prune(cfg: &Config, dry_run: bool) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg)?;

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Prune — Retention Enforcement", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    println!(
        "  {} {} daily, {} weekly, {} monthly",
        crate::theme::style_label("Policy: keep", theme),
        crate::theme::style_value(&cfg.retention.keep_daily.to_string(), theme),
        crate::theme::style_value(&cfg.retention.keep_weekly.to_string(), theme),
        crate::theme::style_value(&cfg.retention.keep_monthly.to_string(), theme),
    );
    if dry_run {
        println!(
            "  {}",
            crate::theme::style_muted("(dry run — no changes)", theme)
        );
    }
    println!();

    // Get all repos
    let mut stmt = conn
        .prepare("SELECT DISTINCT repo_name FROM backups ORDER BY repo_name")
        .context("Failed to query repos")?;
    let repos: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse repo names")?;

    let mut total_pruned = 0u64;
    let mut total_freed: u64 = 0;

    for repo in &repos {
        let mut stmt = conn.prepare(
            "SELECT id, archive_path, size_bytes, created_at FROM backups WHERE repo_name = ?1 ORDER BY created_at DESC"
        ).context("Failed to prepare repo backup query")?;

        let entries: Vec<(i64, String, u64, String)> = stmt
            .query_map([repo], |row| {
                Ok((
                    row.get::<_, i64>("id")?,
                    row.get::<_, String>("archive_path")?,
                    row.get::<_, i64>("size_bytes")? as u64,
                    row.get::<_, String>("created_at")?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to parse entries")?;

        let to_prune = compute_prune_set(&entries, &cfg.retention);

        if to_prune.is_empty() {
            println!(
                "  {} {} — {}",
                crate::theme::style_success("✓", theme),
                crate::theme::style_accent(repo, theme),
                crate::theme::style_muted("nothing to prune", theme),
            );
            continue;
        }

        for (id, archive_path, size, _) in &to_prune {
            println!(
                "  {} #{} {} {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_value(&id.to_string(), theme),
                crate::theme::style_accent(repo, theme),
                crate::theme::style_muted(
                    &format!("({})", crate::utils::format_size(*size)),
                    theme
                ),
            );

            if !dry_run {
                // Delete archive manifest file
                let path = Path::new(archive_path);
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                }
                // Decrement chunk refs and delete backup record
                if let Ok(hashes) = crate::db::get_backup_chunk_hashes(&conn, *id) {
                    for hash in &hashes {
                        let _ = crate::db::decrement_chunk_ref(&conn, hash);
                    }
                }
                conn.execute("DELETE FROM archive_chunks WHERE backup_id = ?1", [*id])?;
                conn.execute("DELETE FROM backups WHERE id = ?1", [*id])?;
            }
            total_freed += size;
            total_pruned += 1;
        }
    }

    println!();
    if total_pruned == 0 {
        println!(
            "  {}",
            crate::theme::style_success(
                "All backups within retention policy. Nothing to prune.",
                theme
            )
        );
    } else {
        println!(
            "  {} {} {} {}",
            crate::theme::style_bold_header(
                &format!("{}{}", if dry_run { "Would prune" } else { "Pruned" }, ":"),
                theme
            ),
            crate::theme::style_value(&total_pruned.to_string(), theme),
            crate::theme::style_muted("backups,", theme),
            crate::theme::style_value(&crate::utils::format_size(total_freed), theme),
            // Note: style_value above returns formatted string, let's just print freed
        );
        println!(
            "     {} {}",
            crate::theme::style_muted("freed:", theme),
            crate::theme::style_value(&crate::utils::format_size(total_freed), theme),
        );
    }

    Ok(())
}

/// Compute which backup entries to prune based on retention policy.
fn compute_prune_set(
    entries: &[(i64, String, u64, String)],
    retention: &crate::config::RetentionConfig,
) -> Vec<(i64, String, u64, String)> {
    if entries.len() <= retention.keep_daily as usize {
        return Vec::new();
    }

    // Simple strategy: keep the most recent N backups, prune the rest.
    // N = keep_daily + keep_weekly + keep_monthly (rough upper bound)
    let keep_count =
        (retention.keep_daily + retention.keep_weekly + retention.keep_monthly) as usize;
    if entries.len() <= keep_count {
        return Vec::new();
    }

    entries[keep_count..].to_vec()
}

// ── forge anvil search (cross-repo code search) ─────────────────────

/// Search across all backed-up repos using ripgrep.
/// Restores each repo to a temp dir and runs rg against them.
pub fn run_search(cfg: &Config, query: &str, limit: usize) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg)?;

    println!(
        "{}",
        crate::theme::style_bold_header(&format!("Forge Anvil Search — \"{}\"", query), theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Check if rg is available
    let rg_available = std::process::Command::new("which")
        .arg("rg")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !rg_available {
        anyhow::bail!("ripgrep (rg) is required for search. Install it with: apt install ripgrep");
    }

    // Get unique repos with their latest backup
    let mut stmt = conn
        .prepare(
            "SELECT id, repo_name, archive_path FROM backups ORDER BY repo_name, created_at DESC",
        )
        .context("Failed to prepare search query")?;

    let entries: Vec<(i64, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>("id")?,
                row.get::<_, String>("repo_name")?,
                row.get::<_, String>("archive_path")?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Deduplicate to latest backup per repo
    let mut seen_repos = HashSet::new();
    let mut latest: Vec<(i64, String, String)> = Vec::new();
    for (id, repo, archive) in entries {
        if seen_repos.insert(repo.clone()) {
            latest.push((id, repo, archive));
        }
    }

    if latest.is_empty() {
        println!(
            "{}",
            crate::theme::style_muted("No backups available for search.", theme)
        );
        return Ok(());
    }

    let mut total_matches = 0usize;

    for (id, repo_name, manifest_path) in &latest {
        let manifest = Path::new(manifest_path);
        if !manifest.exists() {
            continue;
        }

        // Extract to temp dir
        let temp_dir = tempfile::tempdir().context("Failed to create temp dir for search")?;
        let temp_path = match temp_dir.path().to_str() {
            Some(p) => p.to_string(),
            None => {
                println!("  {} {} — {}", crate::theme::style_error("✗", theme), crate::theme::style_accent(repo_name, theme), crate::theme::style_error("non-UTF-8 temp path", theme));
                continue;
            }
        };
        if let Err(e) = crate::archive::extract_dedup_archive(
            cfg,
            manifest_path,
            &temp_path,
        ) {
            println!(
                "  {} {} — {}",
                crate::theme::style_error("✗", theme),
                crate::theme::style_accent(repo_name, theme),
                crate::theme::style_error(&e.to_string(), theme),
            );
            continue;
        }

        // Run ripgrep
        let output = std::process::Command::new("rg")
            .args([
                "--no-heading",
                "--line-number",
                "--color=never",
                "--max-count",
                &limit.to_string(),
                query,
            ])
            .arg(temp_dir.path())
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<&str> = stdout.lines().take(limit).collect();
                if !lines.is_empty() {
                    println!(
                        "\n  {} {}",
                        crate::theme::style_accent(repo_name, theme),
                        crate::theme::style_muted(&format!("(backup #{})", id), theme),
                    );
                    for line in &lines {
                        // Strip the temp path prefix for cleaner output
                        let clean =
                            line.replace(&temp_dir.path().to_string_lossy().to_string(), repo_name);
                        println!("    {}", crate::theme::style_value(&clean, theme),);
                        total_matches += 1;
                    }
                }
            }
            _ => {} // No matches or rg error — skip silently
        }
    }

    println!();
    println!(
        "  {} {}",
        crate::theme::style_bold_header("Total:", theme),
        crate::theme::style_value(
            &format!("{} matches across {} repos", total_matches, latest.len()),
            theme
        ),
    );

    Ok(())
}

// ── forge anvil health (project status) ─────────────────────────────

/// Check the health of all backed-up repos.
pub fn run_health(cfg: &Config) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg)?;

    println!(
        "{}",
        crate::theme::style_bold_header("Forge Anvil Health — Project Status", theme)
    );
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));

    // Get unique repo paths from latest backups
    let mut stmt = conn.prepare(
        "SELECT repo_name, repo_path, branch_count, tag_count, commit_count, backup_type, created_at \
         FROM backups ORDER BY repo_name, created_at DESC"
    ).context("Failed to prepare health query")?;

    let entries: Vec<(String, String, u32, u32, u32, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("repo_name")?,
                row.get::<_, String>("repo_path")?,
                row.get::<_, i64>("branch_count")? as u32,
                row.get::<_, i64>("tag_count")? as u32,
                row.get::<_, i64>("commit_count")? as u32,
                row.get::<_, String>("backup_type")?,
                row.get::<_, String>("created_at")?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse health rows")?;

    // Deduplicate to latest per repo
    let mut seen = HashSet::new();
    let mut latest = Vec::new();
    for entry in entries {
        if seen.insert(entry.0.clone()) {
            latest.push(entry);
        }
    }

    if latest.is_empty() {
        println!("{}", crate::theme::style_muted("No backups found.", theme));
        return Ok(());
    }

    println!(
        "  {:<25} {:<8} {:<8} {:<10} {:<12} {}",
        crate::theme::style_header("Repo", theme),
        crate::theme::style_header("Branches", theme),
        crate::theme::style_header("Tags", theme),
        crate::theme::style_header("Commits", theme),
        crate::theme::style_header("Type", theme),
        crate::theme::style_header("Status", theme),
    );
    println!("  {}", crate::theme::style_border(&"─".repeat(85), theme),);

    for (repo_name, repo_path, branches, tags, commits, backup_type, _created_at) in &latest {
        let path = Path::new(repo_path);
        let status = if !path.exists() {
            crate::theme::style_error("PATH MISSING", theme)
        } else if path.join(".git").exists() {
            // Try to check if it's dirty
            let dirty = std::process::Command::new("git")
                .args(["diff", "--quiet"])
                .current_dir(path)
                .status()
                .map(|s| !s.success())
                .unwrap_or(false);

            let staged = std::process::Command::new("git")
                .args(["diff", "--cached", "--quiet"])
                .current_dir(path)
                .status()
                .map(|s| !s.success())
                .unwrap_or(false);

            if dirty && staged {
                crate::theme::style_error("DIRTY (unstaged+staged)", theme)
            } else if dirty {
                crate::theme::style_error("DIRTY (unstaged)", theme)
            } else if staged {
                crate::theme::style_error("DIRTY (staged)", theme)
            } else {
                crate::theme::style_success("clean", theme)
            }
        } else {
            crate::theme::style_error("NOT A GIT REPO", theme)
        };

        println!(
            "  {:<25} {:<8} {:<8} {:<10} {:<12} {}",
            crate::theme::style_accent(&crate::utils::truncate_str(repo_name, 25), theme),
            crate::theme::style_value(&branches.to_string(), theme),
            crate::theme::style_value(&tags.to_string(), theme),
            crate::theme::style_value(&commits.to_string(), theme),
            crate::theme::style_muted(backup_type.get(..5).unwrap_or(backup_type), theme),
            status,
        );
    }

    println!();
    println!(
        "  {} {} repos checked",
        crate::theme::style_muted("Total:", theme),
        crate::theme::style_value(&latest.len().to_string(), theme),
    );

    Ok(())
}

// ── CLI dispatch for anvil commands ─────────────────────────────────

/// Run the anvil subcommand.
pub fn run_anvil(cfg: &Config, args: &AnvilArgs) -> Result<()> {
    match &args.action {
        Some(AnvilAction::Search { query }) => run_search(cfg, query, 20),
        Some(AnvilAction::Health) => run_health(cfg),
        Some(AnvilAction::Prune { dry_run }) => run_prune(cfg, *dry_run),
        Some(AnvilAction::Verify) => run_temper(cfg),
        Some(AnvilAction::Diff { id1, id2 }) => run_diff(cfg, id1, id2.as_deref()),
        None => {
            // Default: show health
            run_health(cfg)
        }
    }
}

fn run_diff(cfg: &Config, id1: &str, id2: Option<&str>) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);
    let conn = crate::db::connect(cfg)?;

    let backup1 = crate::db::get_backup_by_id(&conn, id1)?
        .ok_or_else(|| anyhow::anyhow!("Backup not found: {}", id1))?;

    let backup2_id = match id2 {
        Some(id) => id.to_string(),
        None => {
            // Find previous backup of same repo
            conn.query_row(
                "SELECT id FROM backups WHERE repo_name = ?1 AND id < ?2 ORDER BY id DESC LIMIT 1",
                rusqlite::params![backup1.repo_name, backup1.id],
                |row| row.get::<_, i64>(0),
            ).ok().map(|id| id.to_string())
            .ok_or_else(|| anyhow::anyhow!("No previous backup found for '{}'", backup1.repo_name))?
        }
    };

    let backup2 = crate::db::get_backup_by_id(&conn, &backup2_id)?
        .ok_or_else(|| anyhow::anyhow!("Backup not found: {}", backup2_id))?;

    // Use tar to list files in each archive for comparison
    fn list_archive_files(archive_path: &std::path::Path) -> Result<std::collections::HashMap<String, String>> {
        let output = std::process::Command::new("tar")
            .args(["tvf", &archive_path.to_string_lossy()])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to list archive: {}", e))?;

        let mut files = std::collections::HashMap::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let path = parts[5..].join(" ");
                let size = parts[2].to_string();
                files.insert(path, size);
            }
        }
        Ok(files)
    }

    let archive1_path = std::path::Path::new(&backup1.archive_path);
    let archive2_path = std::path::Path::new(&backup2.archive_path);

    // Decompress if .zst
    let (path1, cleanup1) = if archive1_path.extension().map_or(false, |e| e == "zst") {
        let decompressed = std::env::temp_dir().join(format!("forge-diff-{}-{}.tar", backup1.id, backup2.id));
        std::process::Command::new("zstd")
            .args(["-d", &archive1_path.to_string_lossy(), "-o", &decompressed.to_string_lossy(), "-f"])
            .status().ok();
        (decompressed, true)
    } else {
        (archive1_path.to_path_buf(), false)
    };

    let (path2, cleanup2) = if archive2_path.extension().map_or(false, |e| e == "zst") {
        let decompressed = std::env::temp_dir().join(format!("forge-diff-2-{}.tar", backup2.id));
        std::process::Command::new("zstd")
            .args(["-d", &archive2_path.to_string_lossy(), "-o", &decompressed.to_string_lossy(), "-f"])
            .status().ok();
        (decompressed, true)
    } else {
        (archive2_path.to_path_buf(), false)
    };

    let files1 = list_archive_files(&path1)?;
    let files2 = list_archive_files(&path2)?;

    if cleanup1 { let _ = std::fs::remove_file(&path1); }
    if cleanup2 { let _ = std::fs::remove_file(&path2); }

    let added: Vec<&String> = files2.keys().filter(|k| !files1.contains_key(*k)).collect();
    let removed: Vec<&String> = files1.keys().filter(|k| !files2.contains_key(*k)).collect();
    let changed: Vec<(&String, &String, &String)> = files1.keys()
        .filter(|k| files2.contains_key(*k) && files1[*k] != files2[*k])
        .map(|k| (k, &files1[k], &files2[k]))
        .collect();

    println!();
    println!("{}", crate::theme::style_bold_header("Forge Anvil — Backup Diff", theme));
    println!("{}", crate::theme::style_border(&"═".repeat(50), theme));
    println!("  {} {} (#{}) vs {} (#{})",
        crate::theme::style_label("Comparing:", theme),
        crate::theme::style_value(&backup1.repo_name, theme),
        crate::theme::style_muted(&backup1.id.to_string(), theme),
        crate::theme::style_value(&backup2.repo_name, theme),
        crate::theme::style_muted(&backup2.id.to_string(), theme),
    );

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        println!("  {} No differences found", crate::theme::style_success("✓", theme));
    }

    if !added.is_empty() {
        println!();
        println!("  {} {} file(s) added", crate::theme::style_success("+", theme), added.len());
        for f in added.iter().take(15) {
            println!("    {} {}", crate::theme::style_success("+", theme), crate::theme::style_value(f, theme));
        }
        if added.len() > 15 {
            println!("    {} {} more...", crate::theme::style_muted("...", theme), added.len() - 15);
        }
    }

    if !removed.is_empty() {
        println!();
        println!("  {} {} file(s) removed", crate::theme::style_error("-", theme), removed.len());
        for f in removed.iter().take(15) {
            println!("    {} {}", crate::theme::style_error("-", theme), crate::theme::style_muted(f, theme));
        }
        if removed.len() > 15 {
            println!("    {} {} more...", crate::theme::style_muted("...", theme), removed.len() - 15);
        }
    }

    if !changed.is_empty() {
        println!();
        println!("  {} {} file(s) changed", crate::theme::style_warning("~", theme), changed.len());
        for (f, s1, s2) in changed.iter().take(10) {
            println!("    {} {} ({} → {})",
                crate::theme::style_warning("~", theme),
                crate::theme::style_value(f, theme),
                crate::theme::style_muted(s1, theme),
                crate::theme::style_value(s2, theme),
            );
        }
        if changed.len() > 10 {
            println!("    {} {} more...", crate::theme::style_muted("...", theme), changed.len() - 10);
        }
    }

    println!();
    Ok(())
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
            llama_swap_config: tmp.path().join("llama-swap-config.yaml"),
        }
    }

    #[test]
    fn compute_prune_set_keeps_recent() {
        let entries: Vec<(i64, String, u64, String)> = (1..=20)
            .map(|i| {
                (
                    i,
                    format!("/archive/{}.manifest.json", i),
                    1024,
                    format!("2026-05-{:02}T00:00:00Z", i),
                )
            })
            .collect();

        let retention = RetentionConfig {
            keep_daily: 3,
            keep_weekly: 2,
            keep_monthly: 2,
        };

        let to_prune = compute_prune_set(&entries, &retention);
        // keep_daily + keep_weekly + keep_monthly = 7, so entries[7..] should be pruned
        assert_eq!(to_prune.len(), 13); // 20 - 7 = 13
        assert_eq!(to_prune[0].0, 8); // First pruned entry is #8
    }

    #[test]
    fn compute_prune_set_nothing_to_prune() {
        let entries: Vec<(i64, String, u64, String)> = (1..=5)
            .map(|i| {
                (
                    i,
                    format!("/archive/{}.manifest.json", i),
                    1024,
                    format!("2026-05-{:02}T00:00:00Z", i),
                )
            })
            .collect();

        let retention = RetentionConfig {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        };

        let to_prune = compute_prune_set(&entries, &retention);
        assert!(to_prune.is_empty());
    }

    #[test]
    fn can_do_incremental_false_when_no_backups() {
        let tmp = TempDir::new().unwrap();
        let cfg = test_config(&tmp);
        let conn = crate::db::connect(&cfg).unwrap();

        assert!(!can_do_incremental(&conn, "nonexistent"));
    }

    #[test]
    fn can_do_incremental_true_after_full_backup() {
        let tmp = TempDir::new().unwrap();
        let cfg = test_config(&tmp);
        let conn = crate::db::connect(&cfg).unwrap();

        let entry = crate::models::BackupEntry {
            id: 0,
            repo_path: "/tmp/test".to_string(),
            repo_name: "test".to_string(),
            archive_path: "/tmp/test.manifest.json".to_string(),
            sha256: "abc".to_string(),
            size_bytes: 100,
            branch_count: 1,
            tag_count: 0,
            commit_count: 5,
            created_at: chrono::Utc::now(),
            backup_type: crate::models::BackupType::Full,
        };

        crate::db::insert_backup(&conn, &entry).unwrap();
        assert!(can_do_incremental(&conn, "test"));
    }
}
