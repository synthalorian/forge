use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn test_config(tmp: &TempDir) -> forge::config::Config {
    forge::config::Config {
        archive_dir: tmp.path().join("archives"),
        db_path: tmp.path().join("forge.db"),
        default_compression: 3,
        repo_paths: vec![],
        retention: forge::config::RetentionConfig {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        theme: "synthwave84".to_string(),
        llama_swap_config: tmp.path().join("llama-swap-config.yaml"),
    }
}

fn create_test_git_repo() -> Result<TempDir> {
    let tmp = TempDir::new()?;

    let out = std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .output()?;
    assert!(
        out.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    std::process::Command::new("git")
        .args(["config", "user.email", "test@forge.test"])
        .current_dir(tmp.path())
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(tmp.path())
        .output()?;

    std::fs::write(tmp.path().join("hello.txt"), "Hello, Forge!")?;
    std::fs::create_dir_all(tmp.path().join("src"))?;
    std::fs::write(tmp.path().join("src/main.rs"), "fn main() {}\n")?;

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()?;

    let out = std::process::Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(tmp.path())
        .output()?;
    assert!(
        out.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    Ok(tmp)
}

#[test]
fn cli_shows_help() {
    Command::cargo_bin("forge")
        .expect("forge binary should exist")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Craft Your Digital Future"));
}

#[test]
fn cli_shows_version() {
    Command::cargo_bin("forge")
        .expect("forge binary should exist")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("forge"));
}

#[test]
fn cli_backup_fails_without_config() {
    let tmp = TempDir::new().expect("temp dir");
    Command::cargo_bin("forge")
        .expect("forge binary should exist")
        .env(
            "XDG_CONFIG_HOME",
            tmp.path().join("config").to_str().expect("path"),
        )
        .env(
            "XDG_DATA_HOME",
            tmp.path().join("data").to_str().expect("path"),
        )
        .arg("backup")
        .assert()
        .failure();
}

#[test]
fn backup_creates_archive_and_db_entry() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: Some(repo_tmp.path().to_str().expect("repo path").to_string()),
            all: false,
            compression: Some(3),
            full: false,
        },
    )?;

    let archive_entries: Vec<_> = std::fs::read_dir(&cfg.archive_dir)?
        .filter_map(|e| e.ok())
        .collect();
    assert!(!archive_entries.is_empty());
    assert!(cfg.archive_dir.join("chunks").exists());

    let conn = forge::db::connect(&cfg)?;
    let entry = forge::db::get_backup_by_id(&conn, "1")?.expect("backup ID 1");
    assert_eq!(entry.id, 1);
    assert_eq!(entry.backup_type, forge::models::BackupType::Full);
    assert!(entry.archive_path.ends_with(".manifest.json"));
    assert!(!entry.sha256.is_empty());
    assert!(entry.branch_count >= 1);

    let chunk_hashes = forge::db::get_backup_chunk_hashes(&conn, entry.id)?;
    assert!(!chunk_hashes.is_empty());

    Ok(())
}

#[test]
fn restore_from_backup() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let forge_tmp = TempDir::new()?;
    let restore_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: Some(repo_tmp.path().to_str().expect("path").to_string()),
            all: false,
            compression: Some(3),
            full: false,
        },
    )?;

    forge::restore::run(
        &cfg,
        &forge::cli::RestoreArgs {
            backup_id: "1".to_string(),
            output: Some(restore_tmp.path().to_str().expect("path").to_string()),
            ref_name: None,
            dry_run: false,
        },
    )?;

    let restored: Vec<_> = std::fs::read_dir(restore_tmp.path())?
        .filter_map(|e| e.ok())
        .collect();
    assert!(!restored.is_empty());
    assert!(restored
        .iter()
        .any(|e| e.file_name().to_string_lossy().ends_with(".git")));

    Ok(())
}

#[test]
fn backup_restore_data_integrity() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let repo_name = repo_tmp
        .path()
        .file_name()
        .expect("dir name")
        .to_str()
        .expect("utf8")
        .to_string();
    let forge_tmp = TempDir::new()?;
    let restore_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: Some(repo_tmp.path().to_str().expect("path").to_string()),
            all: false,
            compression: Some(3),
            full: false,
        },
    )?;

    forge::restore::run(
        &cfg,
        &forge::cli::RestoreArgs {
            backup_id: "1".to_string(),
            output: Some(restore_tmp.path().to_str().expect("path").to_string()),
            ref_name: None,
            dry_run: false,
        },
    )?;

    let bare_dir = restore_tmp.path().join(format!("{}.git", repo_name));
    assert!(bare_dir.exists());
    assert!(bare_dir.join("HEAD").exists());
    assert!(bare_dir.join("objects").exists());
    assert!(bare_dir.join("refs").exists());

    Ok(())
}

#[test]
fn restore_by_name() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let repo_name = repo_tmp
        .path()
        .file_name()
        .expect("dir name")
        .to_str()
        .expect("utf8")
        .to_string();
    let forge_tmp = TempDir::new()?;
    let restore_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: Some(repo_tmp.path().to_str().expect("path").to_string()),
            all: false,
            compression: Some(3),
            full: false,
        },
    )?;

    forge::restore::run(
        &cfg,
        &forge::cli::RestoreArgs {
            backup_id: repo_name,
            output: Some(restore_tmp.path().to_str().expect("path").to_string()),
            ref_name: None,
            dry_run: false,
        },
    )?;

    let restored: Vec<_> = std::fs::read_dir(restore_tmp.path())?
        .filter_map(|e| e.ok())
        .collect();
    assert!(!restored.is_empty());

    Ok(())
}

#[test]
fn restore_dry_run() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: Some(repo_tmp.path().to_str().expect("path").to_string()),
            all: false,
            compression: Some(3),
            full: false,
        },
    )?;

    let restore_tmp = TempDir::new()?;
    forge::restore::run(
        &cfg,
        &forge::cli::RestoreArgs {
            backup_id: "1".to_string(),
            output: Some(restore_tmp.path().to_str().expect("path").to_string()),
            ref_name: None,
            dry_run: true,
        },
    )?;

    let restored: Vec<_> = std::fs::read_dir(restore_tmp.path())?
        .filter_map(|e| e.ok())
        .collect();
    assert!(restored.is_empty());

    Ok(())
}

#[test]
fn multiple_backups_increment_ids() -> Result<()> {
    let repo_tmp = create_test_git_repo()?;
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    let args = forge::cli::BackupArgs {
        path: Some(repo_tmp.path().to_str().expect("path").to_string()),
        all: false,
        compression: Some(3),
        full: false,
    };
    forge::backup::run(&cfg, &args)?;
    forge::backup::run(&cfg, &args)?;

    let conn = forge::db::connect(&cfg)?;
    let first = forge::db::get_backup_by_id(&conn, "1")?.expect("backup 1");
    let second = forge::db::get_backup_by_id(&conn, "2")?.expect("backup 2");
    assert_eq!(first.id, 1);
    assert_eq!(second.id, 2);

    Ok(())
}

// ──────────────────────────────────────────────
// chunkstore module tests
// ──────────────────────────────────────────────

#[test]
fn chunkstore_store_and_verify() -> Result<()> {
    let tmp = TempDir::new()?;
    let chunks_dir = tmp.path().join("chunks");
    let store = forge::chunkstore::ChunkStore::new(chunks_dir, 3)?;

    let data = b"Test chunk data for integration test";
    let info = store.store_chunk(data)?;

    assert!(info.is_new);
    assert!(store.has_chunk(&info.hash));
    assert_eq!(info.original_size, data.len());

    let read_back = store.read_chunk(&info.hash)?;
    assert_eq!(read_back, data.to_vec());

    Ok(())
}

#[test]
fn chunkstore_dedup_not_new() -> Result<()> {
    let tmp = TempDir::new()?;
    let store = forge::chunkstore::ChunkStore::new(tmp.path().join("chunks"), 3)?;

    let data = b"Duplicate me";
    let first = store.store_chunk(data)?;
    assert!(first.is_new);

    let second = store.store_chunk(data)?;
    assert!(!second.is_new);
    assert_eq!(first.hash, second.hash);

    Ok(())
}

#[test]
fn chunkstore_read_back_content_matches() -> Result<()> {
    let tmp = TempDir::new()?;
    let store = forge::chunkstore::ChunkStore::new(tmp.path().join("chunks"), 3)?;

    let data = b"Read-back verification data";
    let info = store.store_chunk(data)?;

    let retrieved = store.read_chunk(&info.hash)?;
    assert_eq!(retrieved, data.to_vec());

    Ok(())
}

#[test]
fn chunkstore_read_nonexistent_fails() -> Result<()> {
    let tmp = TempDir::new()?;
    let store = forge::chunkstore::ChunkStore::new(tmp.path().join("chunks"), 3)?;

    let result = store.read_chunk("0000000000000000000000000000000000000000000000000000000000000000");
    assert!(result.is_err());

    Ok(())
}

// ──────────────────────────────────────────────
// scheduler module tests
// ──────────────────────────────────────────────

#[test]
fn scheduler_validate_cron_valid() -> Result<()> {
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;
    std::fs::create_dir_all(cfg.db_path.parent().unwrap())?;
    std::fs::create_dir_all(&cfg.llama_swap_config.parent().unwrap())?;

    // Create a real directory so the path-exists check passes
    let target_dir = forge_tmp.path().join("myrepo");
    std::fs::create_dir_all(&target_dir)?;

    let result = forge::scheduler::run(
        &cfg,
        &forge::cli::ScheduleArgs {
            action: Some(forge::cli::ScheduleAction::Add {
                cron: "0 9 * * *".to_string(),
                path: target_dir.to_str().expect("path").to_string(),
            }),
        },
    );
    assert!(result.is_ok(), "valid cron should succeed: {:?}", result.err());

    Ok(())
}

#[test]
fn scheduler_validate_cron_invalid() -> Result<()> {
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;
    std::fs::create_dir_all(cfg.db_path.parent().unwrap())?;
    std::fs::create_dir_all(&cfg.llama_swap_config.parent().unwrap())?;

    // Create a real directory so the path-exists check passes
    let target_dir = forge_tmp.path().join("myrepo");
    std::fs::create_dir_all(&target_dir)?;

    let result = forge::scheduler::run(
        &cfg,
        &forge::cli::ScheduleArgs {
            action: Some(forge::cli::ScheduleAction::Add {
                cron: "bad".to_string(),
                path: target_dir.to_str().expect("path").to_string(),
            }),
        },
    );
    assert!(result.is_err(), "invalid cron should fail");

    Ok(())
}

// ──────────────────────────────────────────────
// restore module tests
// ──────────────────────────────────────────────

#[test]
fn restore_fails_invalid_id() -> Result<()> {
    let forge_tmp = TempDir::new()?;
    let cfg = test_config(&forge_tmp);
    std::fs::create_dir_all(&cfg.archive_dir)?;

    let result = forge::restore::run(
        &cfg,
        &forge::cli::RestoreArgs {
            backup_id: "nonexistent".to_string(),
            output: Some(forge_tmp.path().join("out").to_str().expect("path").to_string()),
            ref_name: None,
            dry_run: false,
        },
    );
    assert!(result.is_err());

    Ok(())
}

// ──────────────────────────────────────────────
// backup module tests
// ──────────────────────────────────────────────

#[test]
fn backup_discover_repos_in_dir() -> Result<()> {
    // Create a real git repo in a subdirectory
    let repo_base = TempDir::new()?;
    let repo_dir = repo_base.path().join("myrepo");
    std::fs::create_dir_all(&repo_dir)?;

    let out = std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_dir)
        .output()?;
    assert!(out.status.success(), "git init failed");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@forge.test"])
        .current_dir(&repo_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_dir)
        .output()?;

    std::fs::write(repo_dir.join("readme.md"), "# Test Repo")?;

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_dir)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&repo_dir)
        .output()?;

    let forge_tmp = TempDir::new()?;
    let cfg = forge::config::Config {
        archive_dir: forge_tmp.path().join("archives"),
        db_path: forge_tmp.path().join("forge.db"),
        default_compression: 3,
        repo_paths: vec![repo_base.path().to_str().expect("path").to_string()],
        retention: forge::config::RetentionConfig {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        theme: "synthwave84".to_string(),
        llama_swap_config: forge_tmp.path().join("llama-swap-config.yaml"),
    };
    std::fs::create_dir_all(&cfg.archive_dir)?;

    // When all=true, run calls discover_repos(cfg) which finds repos in repo_paths.
    // This proves the discovery machinery works end-to-end.
    let result = forge::backup::run(
        &cfg,
        &forge::cli::BackupArgs {
            path: None,
            all: true,
            compression: Some(3),
            full: false,
        },
    );
    assert!(result.is_ok(), "backup --all should discover and succeed: {:?}", result.err());

    Ok(())
}
