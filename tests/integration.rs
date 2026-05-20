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
        .stdout(predicate::str::contains("Local git backup"));
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
