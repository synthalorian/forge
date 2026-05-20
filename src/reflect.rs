//! Encrypted prayer journal module for the Forge Spirit subsystem.
//!
//! Uses AES-256-GCM to encrypt journal content at rest in a separate
//! `spirit.db` SQLite database. Key material is stored at
//! `~/.local/share/forge/vault/journal.key` with restricted permissions.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::config::Config;

// ── Structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String,
    pub tag: Option<String>,
    pub word_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryMeta {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub tag: Option<String>,
    pub word_count: u32,
}

// ── Schema ─────────────────────────────────────────────────────────

const SPIRIT_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    encrypted_content BLOB NOT NULL,
    nonce BLOB NOT NULL,
    tag TEXT,
    word_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_entries_created ON journal_entries(created_at DESC);
";

// ── Path helpers ───────────────────────────────────────────────────

/// Derive the data root directory from Config.
/// Since `cfg.db_path` is `{data_root}/forge.db`, the parent is the data root.
fn data_root(cfg: &Config) -> PathBuf {
    cfg.db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/tmp/forge"))
        .to_path_buf()
}

/// Returns `~/.local/share/forge/db/spirit.db` (or test equivalent).
pub fn spirit_db_path(cfg: &Config) -> PathBuf {
    data_root(cfg).join("db").join("spirit.db")
}

fn key_file_path(cfg: &Config) -> PathBuf {
    data_root(cfg).join("vault").join("journal.key")
}

/// Opens or creates `spirit.db` and ensures the schema is migrated.
pub fn connect_spirit_db(cfg: &Config) -> Result<Connection> {
    let path = spirit_db_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create spirit database directory {}",
                parent.display()
            )
        })?;
    }
    let conn = Connection::open(&path)
        .with_context(|| format!("Failed to open spirit database at {}", path.display()))?;
    conn.execute_batch(SPIRIT_SCHEMA)
        .context("Failed to initialize spirit database schema")?;
    Ok(conn)
}

// ── Key management ─────────────────────────────────────────────────

/// Derive or create the journal encryption key.
///
/// Checks for key file at `{data_root}/vault/journal.key`. If not found,
/// generates a new random 32-byte key using `OsRng` and saves it with
/// 0600 permissions on Unix.
pub fn derive_or_create_key(cfg: &Config) -> Result<[u8; 32]> {
    let path = key_file_path(cfg);

    if path.exists() {
        let key_bytes = fs::read(&path)
            .with_context(|| format!("Failed to read journal key from {}", path.display()))?;
        if key_bytes.len() != 32 {
            bail!(
                "Journal key at {} is invalid (expected 32 bytes, got {})",
                path.display(),
                key_bytes.len()
            );
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create vault directory {}", parent.display())
            })?;
        }

        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);

        fs::write(&path, &key)
            .with_context(|| format!("Failed to write journal key to {}", path.display()))?;

        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)
                .with_context(|| format!("Failed to set permissions on {}", path.display()))?;
        }

        Ok(key)
    }
}

// ── Encryption ─────────────────────────────────────────────────────

/// Encrypt plaintext with AES-256-GCM.
///
/// Returns `(ciphertext, 12-byte nonce)`.
pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Result<(Vec<u8>, [u8; 12])> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("Invalid key: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypt ciphertext with AES-256-GCM.
///
/// Returns the original plaintext string.
pub fn decrypt(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 12]) -> Result<String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("Invalid key: {}", e))?;

    let nonce = Nonce::from_slice(nonce);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or corrupted data"))?;

    String::from_utf8(plaintext_bytes).context("Decrypted content is not valid UTF-8")
}

// ── CRUD ───────────────────────────────────────────────────────────

/// Add a new encrypted journal entry. Returns the entry ID.
pub fn add_entry(cfg: &Config, content: &str) -> Result<i64> {
    let key = derive_or_create_key(cfg)?;
    let (ciphertext, nonce) = encrypt(&key, content)?;
    let conn = connect_spirit_db(cfg)?;

    let now = Utc::now().to_rfc3339();
    let word_count = if content.is_empty() {
        0
    } else {
        content.split_whitespace().count() as u32
    };

    conn.execute(
        "INSERT INTO journal_entries (created_at, updated_at, encrypted_content, nonce, tag, word_count)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
        (&now, &now, &ciphertext, &nonce as &[u8], word_count),
    )
    .context("Failed to insert journal entry")?;

    Ok(conn.last_insert_rowid())
}

/// Retrieve a single journal entry by ID, decrypting content.
pub fn get_entry(cfg: &Config, id: i64) -> Result<Option<JournalEntry>> {
    let key = derive_or_create_key(cfg)?;
    let conn = connect_spirit_db(cfg)?;

    let result = conn
        .query_row(
            "SELECT id, created_at, updated_at, encrypted_content, nonce, tag, word_count
             FROM journal_entries WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()
        .context("Failed to query journal entry")?;

    match result {
        Some((id, created_at_str, updated_at_str, encrypted_content, nonce_blob, tag, word_count)) => {
            if nonce_blob.len() != 12 {
                bail!("Invalid nonce length for entry {}", id);
            }
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nonce_blob);

            let content = decrypt(&key, &encrypted_content, &nonce)?;
            let created_at = parse_datetime(&created_at_str)?;
            let updated_at = parse_datetime(&updated_at_str)?;

            Ok(Some(JournalEntry {
                id,
                created_at,
                updated_at,
                content,
                tag,
                word_count: word_count as u32,
            }))
        }
        None => Ok(None),
    }
}

/// List entry metadata (no decryption) for fast browsing.
pub fn list_entries(cfg: &Config, limit: usize, offset: usize) -> Result<Vec<JournalEntryMeta>> {
    let conn = connect_spirit_db(cfg)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, tag, word_count
             FROM journal_entries ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )
        .context("Failed to prepare list query")?;

    let rows: Vec<(i64, String, Option<String>, i64)> = stmt
        .query_map([limit as i64, offset as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .context("Failed to query journal entries")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse journal entry rows")?;

    rows.into_iter()
        .map(|(id, created_at_str, tag, word_count)| {
            Ok(JournalEntryMeta {
                id,
                created_at: parse_datetime(&created_at_str)?,
                tag,
                word_count: word_count as u32,
            })
        })
        .collect()
}

/// Search entries by decrypting all and filtering by content match.
pub fn search_entries(cfg: &Config, query: &str) -> Result<Vec<JournalEntry>> {
    let key = derive_or_create_key(cfg)?;
    let conn = connect_spirit_db(cfg)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, updated_at, encrypted_content, nonce, tag, word_count
             FROM journal_entries ORDER BY created_at DESC",
        )
        .context("Failed to prepare search query")?;

    let rows: Vec<(i64, String, String, Vec<u8>, Vec<u8>, Option<String>, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .context("Failed to query journal entries for search")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to parse search rows")?;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for (id, created_at_str, updated_at_str, encrypted_content, nonce_blob, tag, word_count) in rows
    {
        if nonce_blob.len() != 12 {
            continue;
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_blob);

        let content = match decrypt(&key, &encrypted_content, &nonce) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if content.to_lowercase().contains(&query_lower) {
            results.push(JournalEntry {
                id,
                created_at: parse_datetime(&created_at_str)?,
                updated_at: parse_datetime(&updated_at_str)?,
                content,
                tag,
                word_count: word_count as u32,
            });
        }
    }

    Ok(results)
}

/// Delete a journal entry by ID. Returns `true` if it existed.
pub fn delete_entry(cfg: &Config, id: i64) -> Result<bool> {
    let conn = connect_spirit_db(cfg)?;
    let rows_affected = conn
        .execute("DELETE FROM journal_entries WHERE id = ?1", [id])
        .with_context(|| format!("Failed to delete journal entry {id}"))?;
    Ok(rows_affected > 0)
}

/// Count total journal entries.
pub fn count_entries(cfg: &Config) -> Result<i64> {
    let conn = connect_spirit_db(cfg)?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))
        .context("Failed to count journal entries")?;
    Ok(count)
}

// ── Helpers ────────────────────────────────────────────────────────

fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.to_utc())
        .with_context(|| format!("Failed to parse datetime: {}", s))
}

// ── Tests ──────────────────────────────────────────────────────────

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
        }
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() -> Result<()> {
        let key = [42u8; 32];
        let plaintext = "The Lord is my shepherd; I shall not want.";

        let (ciphertext, nonce) = encrypt(&key, plaintext)?;
        let decrypted = decrypt(&key, &ciphertext, &nonce)?;

        assert_eq!(decrypted, plaintext);
        Ok(())
    }

    #[test]
    fn test_wrong_key_fails() -> Result<()> {
        let key_a = [1u8; 32];
        let key_b = [2u8; 32];
        let plaintext = "secret prayer";

        let (ciphertext, nonce) = encrypt(&key_a, plaintext)?;
        let result = decrypt(&key_b, &ciphertext, &nonce);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_add_and_get_entry() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let id = add_entry(&cfg, "Today I am grateful for peace and quiet.")?;
        assert!(id > 0);

        let entry = get_entry(&cfg, id)?
            .expect("entry should exist");
        assert_eq!(entry.id, id);
        assert_eq!(entry.content, "Today I am grateful for peace and quiet.");
        assert_eq!(entry.word_count, 8);
        assert!(entry.tag.is_none());

        Ok(())
    }

    #[test]
    fn test_list_entries_metadata() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        add_entry(&cfg, "First entry about gratitude.")?;
        add_entry(&cfg, "Second entry about patience.")?;
        add_entry(&cfg, "Third entry about strength.")?;

        let meta = list_entries(&cfg, 10, 0)?;
        assert_eq!(meta.len(), 3);

        for m in &meta {
            assert!(m.id > 0);
            assert!(m.word_count > 0);
        }

        assert!(meta[0].id >= meta[1].id);
        assert!(meta[1].id >= meta[2].id);

        let page = list_entries(&cfg, 2, 0)?;
        assert_eq!(page.len(), 2);

        let offset_page = list_entries(&cfg, 2, 2)?;
        assert_eq!(offset_page.len(), 1);

        Ok(())
    }

    #[test]
    fn test_delete_entry() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let id = add_entry(&cfg, "Entry to be deleted.")?;
        assert!(get_entry(&cfg, id)?.is_some());

        let deleted = delete_entry(&cfg, id)?;
        assert!(deleted);

        assert!(get_entry(&cfg, id)?.is_none());

        let deleted_again = delete_entry(&cfg, id)?;
        assert!(!deleted_again);

        Ok(())
    }

    #[test]
    fn test_empty_content() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let id = add_entry(&cfg, "")?;
        let entry = get_entry(&cfg, id)?.expect("entry should exist");
        assert_eq!(entry.content, "");
        assert_eq!(entry.word_count, 0);

        Ok(())
    }

    #[test]
    fn test_unicode_content() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let content = "🙏 Peace be with you 🕊️ — 世界你好 — 日本語テスト";
        let id = add_entry(&cfg, content)?;
        let entry = get_entry(&cfg, id)?.expect("entry should exist");
        assert_eq!(entry.content, content);

        Ok(())
    }

    #[test]
    fn test_search_entries() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        add_entry(&cfg, "Today I meditated on Psalm 23.")?;
        add_entry(&cfg, "Gratitude list: health, family, purpose.")?;
        add_entry(&cfg, "Reflecting on patience during hardship.")?;
        add_entry(&cfg, "Another psalm reflection — Psalm 91.")?;

        let results = search_entries(&cfg, "psalm")?;
        assert_eq!(results.len(), 2);

        for entry in &results {
            assert!(
                entry.content.to_lowercase().contains("psalm"),
                "Search result should contain query"
            );
        }

        let empty = search_entries(&cfg, "quantum physics")?;
        assert!(empty.is_empty());

        let partial = search_entries(&cfg, "gratitude")?;
        assert_eq!(partial.len(), 1);

        Ok(())
    }

    #[test]
    fn test_count_entries() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        assert_eq!(count_entries(&cfg)?, 0);

        add_entry(&cfg, "one")?;
        add_entry(&cfg, "two")?;
        add_entry(&cfg, "three")?;

        assert_eq!(count_entries(&cfg)?, 3);

        Ok(())
    }

    #[test]
    fn test_get_nonexistent_entry() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        assert!(get_entry(&cfg, 999)?.is_none());
        Ok(())
    }

    #[test]
    fn test_key_derivation_creates_and_reuses() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let key1 = derive_or_create_key(&cfg)?;
        let key2 = derive_or_create_key(&cfg)?;

        assert_eq!(key1, key2);

        let key_path = key_file_path(&cfg);
        assert!(key_path.exists());
        assert_eq!(fs::read(&key_path)?.len(), 32);

        Ok(())
    }

    #[test]
    fn test_spirit_db_is_separate() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = test_config(&tmp);

        let spirit_path = spirit_db_path(&cfg);
        let forge_path = &cfg.db_path;

        assert_ne!(spirit_path, *forge_path);
        assert!(spirit_path.to_string_lossy().contains("spirit.db"));
        assert!(!forge_path.to_string_lossy().contains("spirit.db"));
        assert!(spirit_path.to_string_lossy().contains("/db/spirit.db"));

        Ok(())
    }
}
