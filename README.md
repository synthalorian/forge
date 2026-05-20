```
███████╗ ██████╗ ██████╗ ███████╗
██╔════╝██╔═══██╗██╔══██╗██╔════╝
█████╗  ██║   ██║██████╔╝███████╗
██╔══╝  ██║   ██║██╔══██╗╚════██║
██║     ╚██████╔╝██║  ██║███████║
╚═╝      ╚═════╝ ╚═╝  ╚═╝╚══════╝
```

# forge

**Craft Your Digital Future — from the terminal.**

The blacksmith was the cornerstone of every civilization. They built the tools that built everything else. The sword, the plow, the cathedral — all born at the forge.

Forge is that for the digital age.

A CLI platform where human creativity meets artificial intelligence. Where you don't just store code — you craft your entire digital life. Git backups. AI orchestration. Scripture. System management. Creative tools. All connected. All local. All yours.

One forge to shape them.

---

## The Metaphor

Every blacksmith's workshop has its tools. So does Forge.

```
forge heat      Spin up AI agents — warm the coals
forge strike    Execute a task across agents
forge quench    Save state, backup, cool down
forge temper    Review, refine, improve quality
forge anneal    Deep work mode — slow, methodical, focused
forge alloy     Combine outputs from multiple AI sources
forge cast      Deploy, publish, release into the world
forge grind     Lint, test, sharpen the blade
forge polish    Format, document, make it beautiful
```

---

## The Six Pillars

### I. Forge Code — The Anvil

*Shape your projects with iron precision.*

```
forge anvil init          Scaffold a new project (Unity, Unreal, Godot, Flutter, Rust, Rails)
forge anvil backup        Snapshot git repos — branches, tags, stashes, reflogs, everything
forge anvil restore       Restore from any backup archive
forge anvil search        Search code across all your projects
forge anvil audit         Dependency health check
forge anvil list          List all backups and archives
forge anvil schedule      Automated backup cron management
```

The backup engine is the foundation Forge was built on — zstd compression, SHA-256 content deduplication, SQLite metadata, streaming tar pipelines. It doesn't just copy files. It understands git.

**Multi-Agent Orchestration** — Route coding tasks to the right AI agent automatically. Claude Code for architecture. Codex for rapid prototyping. OpenCode for iterative refactoring. OpenClaw for autonomous workflows. You describe the strike. Forge chooses the hammer.

### II. Forge Mind — The Bellows

*Breathe life into your workflow with AI.*

```
forge breathe              AI agents online — status dashboard
forge breathe spawn        Launch an agent session
forge breathe pipe         Chain agents: research → draft → review → deploy
forge breathe models       List available models (local + cloud)
forge breathe switch       Change active model mid-session
forge breathe prompts      Browse and manage your prompt library
forge breathe vault        Credential management (OAuth, API keys)
```

Inspired by Hermes, oh-my-openagent, and every AI harness that came before — but unified under one roof. Local models via llama-swap. Cloud APIs when you need them. Hybrid when you want both. Your credentials stay in your vault. Your prompts are versioned and shareable. Your agents pick up where they left off.

The bellows don't think. They amplify.

### III. Forge Spirit — The Flame

*The fire that tempers the steel.*

```
forge word                 Today's scripture
forge word search          Verse search and cross-reference
forge word random          A random passage for when you need it
forge reflect              Open your prayer journal
forge reflect entry        Write a new reflection
forge reflect history      Browse past entries
forge rest                 Sabbath mode — shut it all down
```

**"As iron sharpens iron, so one person sharpens another."** — Proverbs 27:17

The forge isn't just about output. It's about the person wielding the hammer. Your prayer journal is encrypted, local, and private — nobody sees it but you and God. Scripture comes from the source, not an API. Sabbath mode shuts down every agent, every process, every daemon — because rest is commanded, not suggested.

**"For we are his workmanship, created in Christ Jesus for good works."** — Ephesians 2:10

The word "workmanship" in the Greek is *poiēma* — the root of "poem." You are a poem being written. Forge helps you write good works with good code.

### IV. Forge System — The Tongs

*Grip and shape your environment.*

```
forge grip                 System status at a glance
forge grip theme           12 built-in themes, custom creation, live preview
forge grip dotfiles        Version, sync, restore your configs
forge grip diagnose        System diagnostics and resource monitoring
forge grip services        Start/stop/restart with dependency awareness
forge grip health          Package health across all projects
```

Omarchy proved that a Linux desktop can be beautiful and functional without touching a GUI config. Forge brings that philosophy to the CLI — theme your terminal, manage your dotfiles, monitor your services, all from one place. The tongs don't care what they're holding. They grip anything.

### V. Forge Create — The Crucible

*Where raw material becomes something beautiful.*

```
forge melt                 Start a creative session
forge melt chords          Generate chord progressions
forge melt image           Generate images from prompts
forge melt diagram         Create ASCII/SVG architecture diagrams
forge melt markdown        Author markdown with live preview
forge melt transcode       Video and audio transcoding
forge melt palette         Generate color palettes
```

The crucible doesn't judge what you put in. Metal, melody, markdown — it melts everything down and lets you reshape it. Music theory helpers for the guitar player. Image generation for the visual thinker. Diagrams for the architect. The creative process doesn't switch apps. It stays at the forge.

### VI. Forge Connect — The Bridge

*Link everything. Human to AI. AI to AI. You to the world.*

```
forge bridge               Check all connections
forge bridge hooks         Manage webhooks
forge bridge notify        Notification hub (Telegram, Discord, Slack, desktop)
forge bridge calendar      Calendar integration
forge bridge sync          Task synchronization across platforms
forge bridge gateway       Expose any local service via API gateway
```

The blacksmith didn't work alone. They were the hub of the village — everyone came to the forge. Connect bridges your tools, your agents, your people, your platforms into one coherent workflow. Webhooks, notifications, calendar events, task sync — all flowing through one pipe.

---

## Architecture

```
~/.forge/
├── config.toml            Your forge configuration
├── vault/                 Encrypted credentials (AES-256)
├── db/
│   ├── forge.db           Core metadata (backups, projects, schedules)
│   ├── spirit.db          Journal entries & scripture bookmarks
│   └── agents.db          AI agent state, sessions, history
├── archives/              Git backups (zstd compressed, content-deduplicated)
├── chunks/                Deduplicated content store (SHA-256 sharded)
├── projects/              Project registry and metadata
├── prompts/               Versioned prompt library
├── themes/                Custom themes (TOML)
├── scripts/               Automation hooks and lifecycle events
└── logs/                  Activity history and audit trail
```

### Archive Format

Each backup produces a `.tar.zst` file containing a bare git clone of the repository. Archives are named `<repo>-<timestamp>.tar.zst`. Metadata (branches, tags, commit count, SHA-256 hash, chunk references) is stored in SQLite for instant querying.

### Content Deduplication

The ChunkStore splits data into 4MB blocks, SHA-256 hashes each one, compresses with zstd, and stores them in a sharded content-addressable layout (`chunks/ab/cdef...zst`). New backups only store chunks they haven't seen before. Across all your projects, this means massive space savings — identical dependencies, assets, and boilerplate are stored once.

---

## Command Reference

```
forge                              Dashboard — what's happening right now
forge init                         First-time setup
forge status                       System & agent health overview

# The Workshop Verbs
forge heat                         Spin up AI agents
forge strike <task>                Execute task via best available agent
forge quench [path]                Backup git repos
forge restore <id>                 Restore from backup
forge temper                       Review recent work, suggest improvements
forge anneal                       Enter deep work mode (do not disturb)
forge alloy <sources>              Merge outputs from multiple agents
forge cast                         Deploy current project
forge grind                        Run linters, tests, quality checks
forge polish                       Format and document

# The Six Pillars
forge anvil <subcommand>           Code & project management
forge breathe <subcommand>         AI agent harness
forge word <subcommand>            Scripture & study
forge reflect <subcommand>         Prayer journal
forge grip <subcommand>            System management
forge melt <subcommand>            Creative tools
forge bridge <subcommand>          Connections & integrations

# Personalization
forge theme list                   Browse themes
forge theme preview <name>         See a theme in action
forge theme set <name>             Apply a theme
forge theme create                 Build your own theme
```

---

## Configuration

Default config at `~/.forge/config.toml`:

```toml
[forge]
name = "my-forge"
data_dir = "~/.forge"

[archive]
compression = 3
chunk_size = 4194304          # 4MB

[retention]
keep_daily = 7
keep_weekly = 4
keep_monthly = 12

[theme]
active = "synthwave84"

[agents]
auto_start = false
preferred = "opencode"        # default agent for forge strike
local_model = "llama-swap"

[spirit]
translation = "ESV"
daily_verse = true
sabbath_mode = false
journal_encrypted = true

[bridge]
notifications = ["desktop"]
webhooks = []
```

---

## Built-in Themes

12 themes, each with 12 color slots. Built for terminals that speak true color.

```
synthwave84        Neon cyan on deep purple — the default
synthwave-night    Magenta and cyan in darkness
synthwave-sunset   Pink and orange horizon
neon-city          Electric blue and hot pink
dark               Clean monochrome for the purist
light              Bright and readable
ocean              Deep blues and seafoam
forest             Greens and earth tones
sunset             Warm oranges and purples
midnight           Dark navy with silver accents
retro              Amber phosphor CRT green
dracula            Purple and green classic
```

Create your own with `forge theme create` — it's just a TOML file with 12 hex colors.

---

## Development Status

**Phase 1 — Foundation (Current)**

| Module | Status |
|--------|--------|
| CLI framework (clap) | Done |
| Configuration system | Done |
| Data models | Done |
| Database schema (SQLite) | Done |
| Backup engine | Done |
| Archive storage (tar+zstd) | Done |
| Restore engine | Done |
| Content deduplication (ChunkStore) | Done |
| Theme engine (12 themes) | Done |
| Cron scheduler | Done |
| Incremental backups | Planned (chunk store ready) |

**Phase 2 — Expansion**

| Module | Status |
|--------|--------|
| AI agent harness | Planned |
| Multi-agent orchestration | Planned |
| Credential vault | Planned |
| Prompt library | Planned |
| Project scaffolding | Planned |
| Scripture module | Planned |
| Prayer journal | Planned |
| Sabbath mode | Planned |
| Dotfile management | Planned |
| System diagnostics | Planned |

**Phase 3 — Vision**

| Module | Status |
|--------|--------|
| Creative tools (music, image, diagrams) | Planned |
| Notification hub | Planned |
| Webhook management | Planned |
| API gateway | Planned |
| Calendar integration | Planned |
| Cross-platform sync | Planned |

---

## Installation

### From source

```bash
git clone https://github.com/synthalorian/forge.git
cd forge
cargo install --path .
```

### Prerequisites

- Rust 1.75+
- Git
- C compiler (for libgit2 and zstd native deps)

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## The Vision

> *Genesis 4:22 — "Tubal-cain, who forged all kinds of tools out of bronze and iron."*

The first craftsman named in Scripture was a forger of tools.
Not a king. Not a priest. A maker.

Forge exists because the terminal is the modern workshop, and the developer is the modern blacksmith. Every line of code is a hammer strike. Every git commit is a cooling blade. Every prayer is fuel for the fire.

We're building the tool that builds all other tools.

The forge doesn't ask permission. It doesn't wait for the cloud. It takes raw material — code, data, ideas, AI output, faith, creativity — and shapes it into something real. Something lasting. Something that outlives the smith.

> *"For we are God's handiwork, created in Christ Jesus to do good works, which God prepared in advance for us to do."* — Ephesians 2:10

The Greek word for "handiwork" is *poiēma*. It's where we get "poem."

You are a poem being written. Write good code. Do good works. Forge your future.

This is the workshop. You are the smith.

**Heat the coals. Strike the iron.** 🔨

---

*Licensed under [Apache 2.0](LICENSE).*

*"The grid remembers everything. So should you."* 🎹🦞
