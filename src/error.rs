use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Not a git repository: {0}")]
    NotGitRepo(String),

    #[error("Backup not found: {0}")]
    BackupNotFound(String),

    #[error("Archive corrupted: {0}")]
    ArchiveCorrupted(String),

    #[error("Schedule not found: {0}")]
    ScheduleNotFound(i64),

    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(String),
}

pub type ForgeResult<T> = Result<T, ForgeError>;
