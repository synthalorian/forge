use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::cli::ListArgs;
use crate::config::Config;
use crate::models::{BackupEntry, BackupType, ScheduleConfig};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS backups (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_path       TEXT NOT NULL,
    repo_name       TEXT NOT NULL,
    archive_path    TEXT NOT NULL,
    sha256          TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL,
    branch_count    INTEGER NOT NULL DEFAULT 0,
    tag_count       INTEGER NOT NULL DEFAULT 0,
    commit_count    INTEGER NOT NULL DEFAULT 0,
    backup_type     TEXT NOT NULL DEFAULT 'full',
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS schedules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    cron_expression TEXT NOT NULL,
    target_path     TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    last_run        TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_backups_repo_name ON backups(repo_name);
CREATE INDEX IF NOT EXISTS idx_backups_created_at ON backups(created_at);

CREATE TABLE IF NOT EXISTS chunks (
    hash                    TEXT PRIMARY KEY,
    original_size           INTEGER NOT NULL,
    compressed_size         INTEGER NOT NULL,
    ref_count               INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS archive_chunks (
    backup_id               INTEGER NOT NULL REFERENCES backups(id),
    chunk_index             INTEGER NOT NULL,
    chunk_hash              TEXT NOT NULL REFERENCES chunks(hash),
    PRIMARY KEY (backup_id, chunk_index)
);
";

pub fn connect(cfg: &Config) -> Result<Connection> {
    let conn = Connection::open(&cfg.db_path)
        .with_context(|| format!("Failed to open database at {}", cfg.db_path.display()))?;
    conn.execute_batch(SCHEMA)
        .context("Failed to initialize database schema")?;
    Ok(conn)
}

fn row_to_backup_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<BackupEntry> {
    let backup_type_str: String = row.get(9)?;
    let created_at_str: String = row.get(10)?;
    Ok(BackupEntry {
        id: row.get(0)?,
        repo_path: row.get(1)?,
        repo_name: row.get(2)?,
        archive_path: row.get(3)?,
        sha256: row.get(4)?,
        size_bytes: row.get::<_, i64>(5)? as u64,
        branch_count: row.get::<_, i64>(6)? as u32,
        tag_count: row.get::<_, i64>(7)? as u32,
        commit_count: row.get::<_, i64>(8)? as u32,
        backup_type: match backup_type_str.as_str() {
            "incremental" => BackupType::Incremental,
            _ => BackupType::Full,
        },
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.to_utc())
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
    })
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

pub fn list_backups(cfg: &Config, args: &ListArgs) -> Result<()> {
    let conn = connect(cfg)?;
    let limit = args.limit.unwrap_or(20);

    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match &args.repo {
        Some(repo) => (
            "SELECT id, repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at
             FROM backups WHERE repo_name = ?1 ORDER BY created_at DESC LIMIT ?2"
                .to_string(),
            vec![Box::new(repo.clone()), Box::new(limit as i64)],
        ),
        None => (
            "SELECT id, repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at
             FROM backups ORDER BY created_at DESC LIMIT ?1"
                .to_string(),
            vec![Box::new(limit as i64)],
        ),
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).context("Failed to prepare list query")?;
    let entries: Vec<BackupEntry> = stmt
        .query_map(param_refs.as_slice(), row_to_backup_entry)
        .context("Failed to execute list query")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse backup rows")?;

    let theme = crate::theme::load_from_config(cfg);

    if args.json {
        let json = serde_json::to_string_pretty(&entries)
            .context("Failed to serialize backups as JSON")?;
        println!("{json}");
    } else if entries.is_empty() {
        println!("{}", crate::theme::style_muted("No backups found.", theme));
    } else {
        println!(
            "{}",
            crate::theme::style_header(
                &format!("{:<5} {:<25} {:<10} {:<6} {:<12} Created", "ID", "Repo", "Branches", "Tags", "Size"),
                theme,
            )
        );
        println!("{}", crate::theme::style_border(&"-".repeat(85), theme));
        for entry in &entries {
            println!(
                "{} {} {} {} {} {}",
                crate::theme::style_value(&format!("{:<5}", entry.id), theme),
                crate::theme::style_accent(&format!("{:<25}", truncate_str(&entry.repo_name, 25)), theme),
                crate::theme::style_value(&format!("{:<10}", entry.branch_count), theme),
                crate::theme::style_value(&format!("{:<6}", entry.tag_count), theme),
                crate::theme::style_value(&format!("{:<12}", format_size(entry.size_bytes)), theme),
                crate::theme::style_value(&entry.created_at.format("%Y-%m-%d %H:%M").to_string(), theme),
            );
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len - 1).collect();
        truncated.push('…');
        truncated
    }
}

pub fn show_status(cfg: &Config) -> Result<()> {
    let conn = connect(cfg)?;
    let theme = crate::theme::load_from_config(cfg);

    let total_backups: i64 = conn
        .query_row("SELECT COUNT(*) FROM backups", [], |row| row.get(0))
        .context("Failed to count backups")?;

    let total_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM backups",
            [],
            |row| row.get(0),
        )
        .context("Failed to sum backup sizes")?;

    let unique_repos: i64 = conn
        .query_row("SELECT COUNT(DISTINCT repo_name) FROM backups", [], |row| {
            row.get(0)
        })
        .context("Failed to count unique repos")?;

    println!("{}", crate::theme::style_bold_header("Forge Backup Status", theme));
    println!("{}", crate::theme::style_border(&"═".repeat(40), theme));
    println!(
        "{}  {}",
        crate::theme::style_label("Total backups:", theme),
        crate::theme::style_value(&total_backups.to_string(), theme),
    );
    println!(
        "{}   {}",
        crate::theme::style_label("Unique repos:", theme),
        crate::theme::style_value(&unique_repos.to_string(), theme),
    );
    println!(
        "{}     {}",
        crate::theme::style_label("Disk usage:", theme),
        crate::theme::style_value(&format_size(total_size as u64), theme),
    );

    if unique_repos > 0 {
        println!();
        println!("{}", crate::theme::style_header("Last backup per repository:", theme));
        println!("{}", crate::theme::style_border(&"-".repeat(60), theme));

        let mut stmt = conn
            .prepare(
                "SELECT repo_name, MAX(created_at) as last_backup FROM backups GROUP BY repo_name ORDER BY repo_name",
            )
            .context("Failed to prepare per-repo status query")?;

        let repo_last: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .context("Failed to query per-repo status")?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("Failed to parse per-repo rows")?;

        for (repo, last) in &repo_last {
            let dt = chrono::DateTime::parse_from_rfc3339(last)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| last.clone());
            println!(
                "  {} {}",
                crate::theme::style_accent(&format!("{:<30}", repo), theme),
                crate::theme::style_value(&dt, theme),
            );
        }
    }

    Ok(())
}

pub fn get_backup_by_id(conn: &Connection, id_or_name: &str) -> Result<Option<BackupEntry>> {
    let sql = "SELECT id, repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at FROM backups";

    if let Ok(numeric_id) = id_or_name.parse::<i64>() {
        let mut stmt = conn
            .prepare(&format!("{sql} WHERE id = ?1"))
            .context("Failed to prepare backup lookup by ID")?;
        let result = stmt
            .query_row([numeric_id], row_to_backup_entry)
            .optional()
            .context("Failed to query backup by ID")?;
        Ok(result)
    } else {
        let mut stmt = conn
            .prepare(&format!(
                "{sql} WHERE repo_name = ?1 ORDER BY created_at DESC LIMIT 1"
            ))
            .context("Failed to prepare backup lookup by name")?;
        let result = stmt
            .query_row([id_or_name], row_to_backup_entry)
            .optional()
            .context("Failed to query backup by name")?;
        Ok(result)
    }
}

pub fn insert_backup(conn: &Connection, entry: &BackupEntry) -> Result<i64> {
    conn.execute(
        "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &entry.repo_path,
            &entry.repo_name,
            &entry.archive_path,
            &entry.sha256,
            entry.size_bytes,
            entry.branch_count,
            entry.tag_count,
            entry.commit_count,
            match entry.backup_type {
                BackupType::Full => "full",
                BackupType::Incremental => "incremental",
            },
            entry.created_at.to_rfc3339(),
        ),
    )
    .context("Failed to insert backup record")?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_schedule(conn: &Connection, schedule: &ScheduleConfig) -> Result<i64> {
    conn.execute(
        "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &schedule.cron_expression,
            &schedule.target_path,
            schedule.enabled,
            schedule.last_run.map(|t| t.to_rfc3339()),
            schedule.created_at.to_rfc3339(),
        ),
    )
    .context("Failed to insert schedule record")?;
    Ok(conn.last_insert_rowid())
}

pub fn list_schedules(conn: &Connection) -> Result<Vec<ScheduleConfig>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, cron_expression, target_path, enabled, last_run, created_at FROM schedules ORDER BY id",
        )
        .context("Failed to prepare schedule list query")?;

    let schedules: Vec<ScheduleConfig> = stmt
        .query_map([], |row| {
            let last_run_str: Option<String> = row.get(4)?;
            let created_at_str: String = row.get(5)?;
            let enabled_int: i64 = row.get(3)?;
            Ok(ScheduleConfig {
                id: row.get(0)?,
                cron_expression: row.get(1)?,
                target_path: row.get(2)?,
                enabled: enabled_int != 0,
                last_run: parse_optional_datetime(last_run_str.as_deref(), 4)?,
                created_at: parse_datetime(&created_at_str, 5)?,
            })
        })
        .context("Failed to query schedules")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse schedule rows")?;

    Ok(schedules)
}

pub fn delete_schedule(conn: &Connection, id: i64) -> Result<()> {
    let rows_affected = conn
        .execute("DELETE FROM schedules WHERE id = ?1", [id])
        .with_context(|| format!("Failed to delete schedule {id}"))?;
    if rows_affected == 0 {
        anyhow::bail!("Schedule with id {id} not found");
    }
    Ok(())
}

fn parse_datetime(s: &str, col: usize) -> rusqlite::Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.to_utc())
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
        })
}

fn parse_optional_datetime(
    s: Option<&str>,
    col: usize,
) -> rusqlite::Result<Option<chrono::DateTime<chrono::Utc>>> {
    match s {
        Some(v) => parse_datetime(v, col).map(Some),
        None => Ok(None),
    }
}

pub fn insert_chunk(
    conn: &Connection,
    hash: &str,
    original_size: u64,
    compressed_size: u64,
) -> Result<()> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE hash = ?1",
            [hash],
            |row| row.get::<_, i64>(0),
        )
        .context("Failed to check chunk existence")?
        > 0;

    if exists {
        conn.execute(
            "UPDATE chunks SET ref_count = ref_count + 1 WHERE hash = ?1",
            [hash],
        )
        .context("Failed to increment chunk ref count")?;
    } else {
        conn.execute(
            "INSERT INTO chunks (hash, original_size, compressed_size, ref_count) VALUES (?1, ?2, ?3, 1)",
            (hash, original_size as i64, compressed_size as i64),
        )
        .context("Failed to insert chunk")?;
    }
    Ok(())
}

pub fn chunk_exists(conn: &Connection, hash: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE hash = ?1",
            [hash],
            |row| row.get(0),
        )
        .context("Failed to check chunk existence")?;
    Ok(count > 0)
}

pub fn increment_chunk_ref(conn: &Connection, hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE chunks SET ref_count = ref_count + 1 WHERE hash = ?1",
        [hash],
    )
    .context("Failed to increment chunk ref count")?;
    Ok(())
}

pub fn decrement_chunk_ref(conn: &Connection, hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE chunks SET ref_count = ref_count - 1 WHERE hash = ?1",
        [hash],
    )
    .context("Failed to decrement chunk ref count")?;
    Ok(())
}

pub fn link_backup_chunks(
    conn: &Connection,
    backup_id: i64,
    chunk_hashes: &[String],
) -> Result<()> {
    for (index, hash) in chunk_hashes.iter().enumerate() {
        conn.execute(
            "INSERT INTO archive_chunks (backup_id, chunk_index, chunk_hash) VALUES (?1, ?2, ?3)",
            (backup_id, index as i64, hash.as_str()),
        )
        .with_context(|| format!("Failed to link chunk {} to backup {}", hash, backup_id))?;
    }
    Ok(())
}

pub fn get_backup_chunk_hashes(conn: &Connection, backup_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT chunk_hash FROM archive_chunks WHERE backup_id = ?1 ORDER BY chunk_index")
        .context("Failed to prepare chunk hash query")?;
    let hashes: Vec<String> = stmt
        .query_map([backup_id], |row| row.get(0))
        .context("Failed to query chunk hashes")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse chunk hash rows")?;
    Ok(hashes)
}
