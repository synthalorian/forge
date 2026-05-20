use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forge")]
#[command(
    version,
    about = "Local git backup & project archive — Time Machine for code"
)]
#[command(after_help = "Run 'forge init' to get started.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize forge configuration")]
    Init,
    #[command(about = "Create a backup of one or more git repositories")]
    Backup(BackupArgs),
    #[command(about = "Restore a repository from a backup archive")]
    Restore(RestoreArgs),
    #[command(about = "List available backups and archives")]
    List(ListArgs),
    #[command(about = "Manage automated backup schedules")]
    Schedule(ScheduleArgs),
    #[command(about = "Manage color themes")]
    Theme(ThemeArgs),
    #[command(about = "Show backup status and statistics")]
    Status,
}

#[derive(Args)]
pub struct BackupArgs {
    #[arg(help = "Path to the git repository to back up")]
    pub path: Option<String>,
    #[arg(short, long, help = "Back up all repositories found in the directory")]
    pub all: bool,
    #[arg(short, long, help = "Compression level (1-22, default 3)")]
    pub compression: Option<u32>,
    #[arg(long, help = "Force a full backup even if incremental is available")]
    pub full: bool,
}

#[derive(Args)]
pub struct RestoreArgs {
    #[arg(help = "ID or name of the backup to restore")]
    pub backup_id: String,
    #[arg(short, long, help = "Target directory for restoration")]
    pub output: Option<String>,
    #[arg(long, help = "Restore a specific branch or tag")]
    pub ref_name: Option<String>,
    #[arg(long, help = "Dry run — show what would be restored")]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, help = "Show only backups for a specific repository")]
    pub repo: Option<String>,
    #[arg(short, long, help = "Maximum number of results")]
    pub limit: Option<usize>,
    #[arg(long, help = "Output as JSON")]
    pub json: bool,
}

#[derive(Args)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub action: Option<ScheduleAction>,
}

#[derive(Subcommand)]
pub enum ScheduleAction {
    #[command(about = "Add a new scheduled backup")]
    Add {
        #[arg(help = "Cron expression for the schedule")]
        cron: String,
        #[arg(help = "Path to the repository or directory")]
        path: String,
    },
    #[command(about = "Remove a scheduled backup")]
    Remove {
        #[arg(help = "ID of the schedule to remove")]
        id: i64,
    },
    #[command(about = "List all configured schedules")]
    List,
}

#[derive(Args)]
pub struct ThemeArgs {
    #[command(subcommand)]
    pub action: ThemeAction,
}

#[derive(Subcommand)]
pub enum ThemeAction {
    #[command(about = "List all available themes")]
    List,
    #[command(about = "Preview a specific theme (default: current theme)")]
    Preview {
        #[arg(help = "Theme name to preview")]
        name: Option<String>,
    },
    #[command(about = "Set the active theme")]
    Set {
        #[arg(help = "Name of the theme to activate")]
        name: String,
    },
}
