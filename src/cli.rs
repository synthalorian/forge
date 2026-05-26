use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forge")]
#[command(version, about = "Craft Your Digital Future — from the terminal")]
#[command(
    after_help = "Run 'forge init' to get started. Run 'forge' with no args for the dashboard."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize forge configuration")]
    Init,
    #[command(
        about = "Create a backup of one or more git repositories",
        alias = "quench",
        alias = "q"
    )]
    Backup(BackupArgs),
    #[command(about = "Restore a repository from a backup archive")]
    Restore(RestoreArgs),
    #[command(about = "List available backups and archives", alias = "l")]
    List(ListArgs),
    #[command(about = "Manage automated backup schedules")]
    Schedule(ScheduleArgs),
    #[command(about = "Manage color themes")]
    Theme(ThemeArgs),
    #[command(about = "Show backup status and statistics")]
    Status,
    #[command(about = "Scripture & study")]
    Word(WordArgs),
    #[command(about = "Prayer journal")]
    Reflect(ReflectArgs),
    #[command(about = "Sabbath mode — shut it all down")]
    Rest,
    #[command(about = "AI agent status dashboard")]
    Breathe(BreatheArgs),
    #[command(about = "Execute a task via best available AI agent", alias = "s")]
    Strike(StrikeArgs),
    #[command(about = "Verify backup integrity (re-hash & compare)")]
    Temper,
    #[command(about = "Anvil commands — search, health, prune, verify")]
    Anvil(AnvilArgs),
    #[command(about = "System resource dashboard")]
    Grip(GripArgs),
    #[command(about = "Creative tools — chords, palettes, diagrams")]
    Melt(MeltArgs),
    #[command(about = "Connection status & integration hub")]
    Bridge(BridgeArgs),
    #[command(about = "Spin up AI agents")]
    Heat,
    #[command(about = "Deep work mode — do not disturb")]
    Anneal,
    #[command(about = "Merge outputs from multiple agents")]
    Alloy,
    #[command(about = "Deploy current project")]
    Cast,
    #[command(about = "Run linters, tests, quality checks")]
    Grind,
    #[command(about = "Format and document")]
    Polish,
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
    #[command(about = "Create a custom theme interactively")]
    Create,
    #[command(about = "Export a theme to terminal config format (Alacritty/Kitty/Ghostty)")]
    Export {
        #[arg(help = "Theme name to export (default: active theme)")]
        name: Option<String>,
        #[arg(short, long, help = "Export format: alacritty, kitty, ghostty")]
        format: Option<String>,
        #[arg(short, long, help = "Write directly to terminal config file")]
        write: bool,
    },
}

#[derive(Args)]
pub struct WordArgs {
    #[command(subcommand)]
    pub action: Option<WordAction>,
}

#[derive(Subcommand)]
pub enum WordAction {
    #[command(about = "Today's verse (default if no subcommand)")]
    Daily,
    #[command(about = "Search scripture")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
    #[command(about = "Look up a passage")]
    Reference {
        #[arg(help = "Book name")]
        book: String,
        #[arg(short, long, help = "Chapter number")]
        chapter: Option<u32>,
        #[arg(short, long, help = "Verse number")]
        verse: Option<u32>,
    },
}

#[derive(Args)]
pub struct ReflectArgs {
    #[command(subcommand)]
    pub action: Option<ReflectAction>,
}

#[derive(Subcommand)]
pub enum ReflectAction {
    #[command(about = "Write a journal entry")]
    Entry {
        #[arg(help = "Journal entry text")]
        text: String,
    },
    #[command(about = "Browse past entries")]
    History,
    #[command(about = "Read a specific entry")]
    Read {
        #[arg(help = "Entry ID")]
        id: i64,
    },
    #[command(about = "Search entries")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
}

#[derive(Args)]
pub struct BreatheArgs {
    #[command(subcommand)]
    pub action: Option<BreatheAction>,
}

#[derive(Subcommand)]
pub enum BreatheAction {
    #[command(about = "Show agent status (default)")]
    Status,
    #[command(about = "List available AI models")]
    Models,
    #[command(about = "Manage credentials")]
    Vault,
    #[command(about = "Manage prompt library")]
    Prompts,
    #[command(about = "Run a multi-step agent pipeline from a TOML definition")]
    Pipe {
        #[arg(help = "Path to the pipeline TOML file")]
        path: String,
    },
    #[command(about = "Manage agent conversation sessions")]
    Sessions {
        #[command(subcommand)]
        action: Option<SessionAction>,
    },
}

#[derive(Subcommand)]
pub enum SessionAction {
    #[command(about = "List all agent sessions")]
    List {
        #[arg(short, long, help = "Filter by agent name")]
        agent: Option<String>,
        #[arg(short, long, help = "Maximum results (default 20)")]
        limit: Option<usize>,
    },
    #[command(about = "Show session details and messages")]
    Show {
        #[arg(help = "Session ID")]
        id: i64,
    },
    #[command(about = "Delete a session")]
    Delete {
        #[arg(help = "Session ID")]
        id: i64,
    },
    #[command(about = "Create a new session")]
    Create {
        #[arg(help = "Agent name")]
        agent: String,
        #[arg(help = "Initial message")]
        message: Option<String>,
    },
}

#[derive(Args)]
pub struct StrikeArgs {
    #[arg(help = "Task description to delegate")]
    pub task: String,
    #[arg(short, long, help = "Force a specific agent")]
    pub agent: Option<String>,
}

#[derive(Args)]
pub struct AnvilArgs {
    #[command(subcommand)]
    pub action: Option<AnvilAction>,
}

#[derive(Subcommand)]
pub enum AnvilAction {
    #[command(about = "Search across all backed-up repos")]
    Search {
        #[arg(help = "Search query (ripgrep pattern)")]
        query: String,
    },
    #[command(about = "Show project health status")]
    Health,
    #[command(about = "Enforce retention policy")]
    Prune {
        #[arg(long, help = "Dry run — show what would be pruned")]
        dry_run: bool,
    },
    #[command(about = "Verify backup integrity")]
    Verify,
}

#[derive(Args)]
pub struct GripArgs {
    #[command(subcommand)]
    pub action: Option<GripAction>,
}

#[derive(Subcommand)]
pub enum GripAction {
    #[command(about = "System resource dashboard")]
    Dashboard,
    #[command(about = "Track and version dotfiles")]
    Dotfiles {
        #[command(subcommand)]
        action: Option<DotfilesAction>,
    },
    #[command(about = "System health check")]
    Diagnose,
    #[command(about = "Install or list git hooks for auto-backup on commit")]
    Hooks {
        #[command(subcommand)]
        action: Option<HooksAction>,
    },
}

#[derive(Subcommand)]
pub enum HooksAction {
    #[command(about = "Install forge post-commit hook in current repo")]
    Install,
    #[command(about = "List installed forge hooks in current repo")]
    List,
}

#[derive(Subcommand)]
pub enum DotfilesAction {
    #[command(about = "Track a dotfile")]
    Track {
        #[arg(help = "Path to the file to track")]
        path: String,
    },
    #[command(about = "List tracked dotfiles")]
    List,
    #[command(about = "Restore a tracked dotfile")]
    Restore {
        #[arg(help = "Name of the dotfile to restore")]
        name: Option<String>,
    },
}

#[derive(Args)]
pub struct MeltArgs {
    #[command(subcommand)]
    pub action: MeltAction,
}

#[derive(Subcommand)]
pub enum MeltAction {
    #[command(about = "Generate chord progressions")]
    Chords {
        #[arg(help = "Key (e.g., C, Am, G)")]
        key: Option<String>,
        #[arg(short, long, help = "Scale type: major, minor, dorian, mixolydian")]
        scale: Option<String>,
        #[arg(
            short,
            long,
            help = "Suggest by mood: happy, sad, epic, chill, worship"
        )]
        mood: Option<String>,
    },
    #[command(about = "Generate color palettes")]
    Palette {
        #[arg(help = "Base color (hex like #FF6B9D or named)")]
        color: Option<String>,
        #[arg(
            short,
            long,
            help = "Harmony type: complementary, analogous, triadic, split"
        )]
        harmony: Option<String>,
        #[arg(short, long, help = "Export format: terminal, css, tailwind")]
        format: Option<String>,
        #[arg(short, long, help = "Extract palette from an image file")]
        file: Option<String>,
    },
    #[command(about = "Generate ASCII/SVG diagrams")]
    Diagram {
        #[arg(help = "Diagram type: flow, sequence, architecture")]
        diag_type: Option<String>,
        #[arg(short, long, help = "Diagram description")]
        description: Option<String>,
    },
    #[command(about = "Render markdown to terminal with formatting")]
    Markdown {
        #[arg(help = "Path to markdown file (use '-' for stdin)")]
        path: Option<String>,
    },
    #[command(about = "Generate an abstract image from a text prompt")]
    Image {
        #[arg(help = "Description of the image to generate")]
        prompt: String,
        #[arg(short, long, help = "Output width in pixels (default: 800)")]
        width: Option<u32>,
        #[arg(short, long, help = "Output height in pixels (default: 600)")]
        height: Option<u32>,
        #[arg(short, long, help = "Output path (default: ~/.forge/images/)")]
        output: Option<String>,
    },
    #[command(about = "Generate L-system fractals (Koch, Dragon, Sierpinski, Plant)")]
    Fractal {
        #[arg(help = "Named preset or custom axiom. Presets: koch, dragon, sierpinski, plant")]
        preset: Option<String>,
        #[arg(long, help = "Custom axiom string (overrides preset)")]
        axiom: Option<String>,
        #[arg(long, help = "Production rule(s) in format: X→Y+F, Y→FX (comma-separated)")]
        rule: Option<String>,
        #[arg(long, default_value = "4", help = "Number of iterations (1-8)")]
        iterations: Option<usize>,
        #[arg(long, default_value = "90", help = "Turn angle in degrees")]
        angle: Option<f64>,
        #[arg(long, default_value = "ascii", help = "Output format: ascii, svg")]
        output: Option<String>,
    },
}

#[derive(Args)]
pub struct BridgeArgs {
    #[command(subcommand)]
    pub action: Option<BridgeAction>,
}

#[derive(Subcommand)]
pub enum BridgeAction {
    #[command(about = "Show connection status for all integrations")]
    Status,
    #[command(about = "List webhook endpoints")]
    Hooks,
    #[command(about = "Send test notification")]
    Notify {
        #[arg(short, long, help = "Channel: desktop, telegram, discord")]
        channel: Option<String>,
        #[arg(help = "Message to send")]
        message: Option<String>,
    },
    #[command(about = "Sync and display task state across platforms")]
    Sync {
        #[arg(short, long, help = "Show verbose details")]
        verbose: bool,
    },
}
