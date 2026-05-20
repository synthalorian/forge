use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::BackupArgs;
use crate::config::Config;
use crate::models::{BackupEntry, BackupType, RepoSnapshot};

/// Run the backup command.
///
/// Creates a snapshot of the target git repository including all branches,
/// tags, stashes, and reflogs. Produces a compressed archive stored in the
/// configured archive directory and records metadata in the SQLite database.
pub fn run(cfg: &Config, args: &BackupArgs) -> Result<()> {
    let compression = args.compression.unwrap_or(cfg.default_compression);

    if args.all {
        run_all(cfg, compression)
    } else {
        let repo_path = match &args.path {
            Some(p) => PathBuf::from(p),
            None => std::env::current_dir().context("Failed to get current directory")?,
        };
        run_single(cfg, &repo_path, compression)
    }
}

fn run_all(cfg: &Config, compression: u32) -> Result<()> {
    let repos = discover_repos(cfg)?;
    if repos.is_empty() {
        anyhow::bail!("No git repositories found in configured paths or current directory");
    }

    let theme = crate::theme::load_from_config(cfg);

    let pb = ProgressBar::new(repos.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
        )
        .context("Failed to create progress style")?
        .progress_chars("#>-"),
    );

    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for repo_path in &repos {
        pb.set_message(format!("Backing up {}", repo_path.display()));
        match backup_repo(cfg, repo_path, compression) {
            Ok(info) => {
                pb.println(format!(
                    "  {} {} {} {} ({} {}, {} {}, {} {}, {} {})",
                    crate::theme::style_success("✓", theme),
                    crate::theme::style_accent(&info.repo_name, theme),
                    crate::theme::style_muted("—", theme),
                    crate::theme::style_value(&format_size(info.size_bytes), theme),
                    crate::theme::style_value(&info.branch_count.to_string(), theme),
                    crate::theme::style_muted("branches", theme),
                    crate::theme::style_value(&info.tag_count.to_string(), theme),
                    crate::theme::style_muted("tags", theme),
                    crate::theme::style_value(&info.total_chunks.to_string(), theme),
                    crate::theme::style_muted("chunks", theme),
                    crate::theme::style_value(&info.new_chunks.to_string(), theme),
                    crate::theme::style_muted("new", theme),
                ));
                succeeded += 1;
            }
            Err(e) => {
                pb.println(format!(
                    "  {} {}: {}",
                    crate::theme::style_error("✗", theme),
                    crate::theme::style_muted(&repo_path.display().to_string(), theme),
                    crate::theme::style_error(&e.to_string(), theme),
                ));
                failed += 1;
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message(format!(
        "{} {} {}, {} {} {} {}",
        crate::theme::style_bold_header("Done:", theme),
        crate::theme::style_value(&succeeded.to_string(), theme),
        crate::theme::style_muted("succeeded", theme),
        crate::theme::style_value(&failed.to_string(), theme),
        crate::theme::style_muted("failed", theme),
        crate::theme::style_muted("out of", theme),
        crate::theme::style_value(&repos.len().to_string(), theme),
    ));

    Ok(())
}

fn run_single(cfg: &Config, repo_path: &Path, compression: u32) -> Result<()> {
    let theme = crate::theme::load_from_config(cfg);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
            .context("Failed to create progress style")?,
    );
    pb.set_message(format!("Backing up {}", repo_path.display()));

    let info = backup_repo(cfg, repo_path, compression)?;

    pb.finish_with_message(format!(
        "{} {} {}: {} ({} {}, {} {}, {} {}, {} {})",
        crate::theme::style_success("✓", theme),
        crate::theme::style_muted("Backed up", theme),
        crate::theme::style_accent(&info.repo_name, theme),
        crate::theme::style_value(&format_size(info.size_bytes), theme),
        crate::theme::style_value(&info.branch_count.to_string(), theme),
        crate::theme::style_muted("branches", theme),
        crate::theme::style_value(&info.tag_count.to_string(), theme),
        crate::theme::style_muted("tags", theme),
        crate::theme::style_value(&info.total_chunks.to_string(), theme),
        crate::theme::style_muted("chunks", theme),
        crate::theme::style_value(&info.new_chunks.to_string(), theme),
        crate::theme::style_muted("new", theme),
    ));

    Ok(())
}

struct BackupInfo {
    repo_name: String,
    size_bytes: u64,
    branch_count: u32,
    tag_count: u32,
    total_chunks: u64,
    new_chunks: u64,
}

fn backup_repo(cfg: &Config, repo_path: &Path, compression: u32) -> Result<BackupInfo> {
    let canonical = repo_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path {}", repo_path.display()))?;

    let mut repo = git2::Repository::open(&canonical)
        .with_context(|| format!("Failed to open git repo at {}", canonical.display()))?;

    let repo_name = canonical
        .file_name()
        .context("Cannot extract repo name from path")?
        .to_str()
        .context("Repo name is not valid UTF-8")?
        .to_string();

    let repo_path_str = canonical
        .to_str()
        .context("Repo path is not valid UTF-8")?
        .to_string();

    let branches = collect_branches(&repo)?;
    let branch_count = branches.len() as u32;

    let tags = collect_tags(&mut repo)?;
    let tag_count = tags.len() as u32;

    let stash_count = count_stashes(&mut repo)?;

    let head_commit = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string())
        .unwrap_or_default();

    let is_dirty = is_repo_dirty(&repo)?;
    let commit_count = count_commits(&repo)?;

    let snapshot = RepoSnapshot {
        repo_name: repo_name.clone(),
        repo_path: repo_path_str,
        branches,
        tags,
        stash_count,
        head_commit,
        is_dirty,
        captured_at: chrono::Utc::now(),
    };

    let tmp_dir =
        std::env::temp_dir().join(format!("forge-bare-{}-{}", repo_name, std::process::id(),));

    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("Failed to create temp dir {}", tmp_dir.display()))?;

    let bare_path = tmp_dir.join(format!("{}.git", repo_name));
    let bare_str = bare_path.to_str().context("Temp path is not valid UTF-8")?;

    let clone_status = std::process::Command::new("git")
        .args(["clone", "--bare"])
        .arg(&canonical)
        .arg(bare_str)
        .status()
        .context("Failed to spawn git clone --bare")?;

    if !clone_status.success() {
        let _ = fs::remove_dir_all(&tmp_dir);
        anyhow::bail!(
            "git clone --bare failed with exit code {:?}",
            clone_status.code(),
        );
    }

    let dedup_result = crate::archive::create_dedup_archive(cfg, bare_str, compression)
        .with_context(|| format!("Failed to create dedup archive for {}", bare_path.display()))?;

    if let Err(e) = fs::remove_dir_all(&tmp_dir) {
        tracing::warn!("Failed to clean up {}: {}", tmp_dir.display(), e);
    }

    let conn = crate::db::connect(cfg)?;
    let entry = BackupEntry {
        id: 0,
        repo_path: snapshot.repo_path.clone(),
        repo_name: snapshot.repo_name.clone(),
        archive_path: dedup_result.manifest_path.clone(),
        sha256: dedup_result.manifest_sha256.clone(),
        size_bytes: dedup_result.compressed_size,
        branch_count,
        tag_count,
        commit_count,
        backup_type: BackupType::Full,
        created_at: snapshot.captured_at,
    };
    let backup_id =
        crate::db::insert_backup(&conn, &entry).context("Failed to insert backup record")?;

    // Insert chunk records into DB before linking (satisfies FK constraint)
    for chunk in &dedup_result.chunk_details {
        crate::db::insert_chunk(&conn, &chunk.hash, chunk.original_size, chunk.compressed_size)
            .with_context(|| format!("Failed to insert chunk {} into database", chunk.hash))?;
    }

    crate::db::link_backup_chunks(&conn, backup_id, &dedup_result.chunk_hashes)
        .context("Failed to link backup chunks")?;

    Ok(BackupInfo {
        repo_name,
        size_bytes: dedup_result.compressed_size,
        branch_count,
        tag_count,
        total_chunks: dedup_result.total_chunks,
        new_chunks: dedup_result.new_chunks,
    })
}

fn collect_branches(repo: &git2::Repository) -> Result<Vec<String>> {
    let mut branches = Vec::new();
    for bt in [git2::BranchType::Local, git2::BranchType::Remote] {
        let iter = repo
            .branches(Some(bt))
            .with_context(|| format!("Failed to enumerate {:?} branches", bt))?;
        for branch_result in iter {
            let (branch, _) = branch_result.context("Failed to read branch entry")?;
            if let Some(name) = branch.name().context("Failed to read branch name")? {
                branches.push(name.to_string());
            }
        }
    }
    Ok(branches)
}

fn collect_tags(repo: &mut git2::Repository) -> Result<Vec<String>> {
    let mut tags = Vec::new();
    repo.tag_foreach(|_id, name_bytes| {
        if let Ok(full_name) = std::str::from_utf8(name_bytes) {
            let tag_name = full_name.strip_prefix("refs/tags/").unwrap_or(full_name);
            tags.push(tag_name.to_string());
        }
        true
    })
    .context("Failed to enumerate tags")?;
    Ok(tags)
}

fn count_stashes(repo: &mut git2::Repository) -> Result<u32> {
    let mut count: u32 = 0;
    repo.stash_foreach(|_, _, _| {
        count += 1;
        true
    })
    .context("Failed to enumerate stashes")?;
    Ok(count)
}

/// Checks both unstaged (index vs workdir) and staged (HEAD vs index) changes.
fn is_repo_dirty(repo: &git2::Repository) -> Result<bool> {
    let diff_unstaged = repo
        .diff_index_to_workdir(None, None)
        .context("Failed to diff index against working directory")?;
    if diff_unstaged.deltas().len() > 0 {
        return Ok(true);
    }

    if let Ok(head_ref) = repo.head() {
        if let Ok(head_tree) = head_ref.peel_to_tree() {
            let diff_staged = repo
                .diff_tree_to_index(Some(&head_tree), None, None)
                .context("Failed to diff HEAD against index")?;
            if diff_staged.deltas().len() > 0 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Uses first-parent simplification for speed on large repositories.
fn count_commits(repo: &git2::Repository) -> Result<u32> {
    let head_ref = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(0),
    };
    let head_oid = match head_ref.target() {
        Some(oid) => oid,
        None => return Ok(0),
    };

    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;
    revwalk
        .push(head_oid)
        .context("Failed to push HEAD to revwalk")?;
    revwalk
        .simplify_first_parent()
        .context("Failed to set first-parent mode")?;

    Ok(revwalk.count() as u32)
}

fn discover_repos(cfg: &Config) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    let mut search_paths: Vec<PathBuf> = cfg.repo_paths.iter().map(PathBuf::from).collect();

    if let Ok(cwd) = std::env::current_dir() {
        search_paths.push(cwd);
    }

    for base in &search_paths {
        if !base.exists() {
            continue;
        }

        if base.join(".git").exists() {
            if let Ok(canonical) = base.canonicalize() {
                if seen.insert(canonical.clone()) {
                    repos.push(canonical);
                }
            }
        }

        for entry in walkdir::WalkDir::new(base)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                let name = e.file_name().to_str().unwrap_or("");
                if name == ".git" || name == "node_modules" || name == "target" {
                    return false;
                }
                e.file_type().is_dir()
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().is_dir() && entry.path().join(".git").exists() {
                if let Ok(canonical) = entry.path().canonicalize() {
                    if seen.insert(canonical.clone()) {
                        repos.push(canonical);
                    }
                }
            }
        }
    }

    Ok(repos)
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
