use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: i64,
    pub repo_path: String,
    pub repo_name: String,
    pub archive_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub branch_count: u32,
    pub tag_count: u32,
    pub commit_count: u32,
    pub created_at: DateTime<Utc>,
    pub backup_type: BackupType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub repo_name: String,
    pub repo_path: String,
    pub branches: Vec<String>,
    pub tags: Vec<String>,
    pub stash_count: u32,
    pub head_commit: String,
    pub is_dirty: bool,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub backup_id: i64,
    pub repo_snapshot: RepoSnapshot,
    pub archive_path: String,
    pub compression_level: u32,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub dedup_chunks_total: u64,
    pub dedup_chunks_new: u64,
    pub chunk_hashes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkEntry {
    pub hash: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub ref_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub id: i64,
    pub cron_expression: String,
    pub target_path: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
