<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/forge-icon.png">
  <img src="assets/forge-icon.png" alt="Forge" width="600">
</picture>

# Forge

**Craft Your Digital Future — from the terminal.**

The blacksmith was the cornerstone of every civilization. They built the tools that built everything else. The sword, the plow, the cathedral — all born at the forge.

Forge is that for the digital age.

A CLI platform where human creativity meets artificial intelligence. Where you don't just store code — you craft your entire digital life. Git backups. AI orchestration. Scripture. System management. Creative tools. All connected. All local. All yours.

**One forge to shape them.**

---

## Features

### ✅ Core — The Anvil (Phase 1 — Done)
- **Git backup engine** — Full bare clone with zstd compression and SHA-256 content dedup
- **Restore engine** — Extract from archive, optional ref checkout, dry-run
- **SQLite metadata** — Instant querying of backups, schedules, archives, chunks
- **Cron scheduler** — Set-and-forget automated backups via crontab
- **ChunkStore** — Content-addressable 4MB blocks, dedup across all projects
- **12 built-in themes** — synthwave84, dark, ocean, forest, sunset, midnight, retro, dracula and more
- **Theme engine** — True color, 12 color slots, live preview, set as default
- **Streaming archive pipeline** — tar → zstd → SHA-256, no temp files

### 🔮 Spirit — The Flame (Phase 2 — In Progress)
- **`forge word`** — Daily scripture, verse search, passage reference lookup
- **`forge reflect`** — Encrypted prayer journal with AES-256-GCM
- **`forge rest`** — Sabbath mode: shut down all agents and processes
- Bundled KJV Bible as SQLite — zero network dependency

### 🧠 Mind — The Bellows (Phase 2 — Planned)
- **`forge breathe`** — Agent status dashboard (Hermes, llama-swap, OpenCode)
- **`forge strike <task>`** — Route tasks to best available AI agent
- **`forge breathe models`** — List local + cloud models
- **`forge breathe vault`** — Credential management (OAuth, API keys)
- **`forge breathe prompts`** — TOML-based prompt library CRUD
- Session persistence in SQLite

### 🔧 System — The Tongs (Phase 2 — Planned)
- **`forge grip`** — System dashboard (CPU, memory, disk, GPU)
- **`forge grip dotfiles`** — Version and restore dotfiles
- **`forge grip services`** — Running service monitor
- **`forge grip diagnose`** — System health check
- **`forge theme create`** — Interactive theme builder
- **`forge theme export`** — Export to Alacritty, Kitty, Ghostty formats

### 🎨 Create — The Crucible (Phase 3 — Vision)
- **`forge melt chords`** — Chord progressions and music theory helpers
- **`forge melt image`** — Bridge to image generation
- **`forge melt diagram`** — ASCII and SVG architecture diagrams
- **`forge melt palette`** — Color palette generation (from scratch or from images)
- **`forge melt markdown`** — Markdown authoring with preview

### 🌉 Connect — The Bridge (Phase 3 — Vision)
- **`forge bridge`** — Check all connection statuses
- **`forge bridge hooks`** — Webhook endpoint management
- **`forge bridge notify`** — Notification hub (Telegram, Discord, desktop)
- **`forge bridge sync`** — Cross-platform task synchronization
- **`forge bridge calendar`** — Calendar integration

---

## Quick Start

```bash
# Install from source
git clone https://github.com/synthalorian/forge.git
cd forge
cargo install --path .

# Initialize
forge init

# Backup a project
forge quench /path/to/project

# See your backups
forge list

# Set the theme
forge theme set synthwave84
```

### Prerequisites

- **Rust 1.75+** (stable)
- **Git** (for backup engine)
- **C compiler** (for native dependencies: libgit2, zstd)

---

## Command Reference

```
forge                              Dashboard — what's happening right now
forge init                         First-time setup
forge status                       System & backup health overview

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

## Architecture

```
~/.forge/
├── config.toml            Your forge configuration
├── vault/                 Encrypted credentials (AES-256-GCM)
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

## Built-in Themes

Forge comes with 12 themes, each with 12 color slots. Built for terminals that speak true color.

| Theme | Description |
|-------|-------------|
| `synthwave84` | Neon cyan on deep purple — the default |
| `synthwave-night` | Magenta and cyan in darkness |
| `synthwave-sunset` | Pink and orange horizon |
| `neon-city` | Electric blue and hot pink |
| `dark` | Clean monochrome for the purist |
| `light` | Bright and readable |
| `ocean` | Deep blues and seafoam |
| `forest` | Greens and earth tones |
| `sunset` | Warm oranges and purples |
| `midnight` | Dark navy with silver accents |
| `retro` | Amber phosphor CRT green |
| `dracula` | Purple and green classic |

Create your own with `forge theme create` — it's just a TOML file with 12 hex colors.

---

## Development Status

### Phase 1 — Foundation ✓

| Module | Status |
|--------|--------|
| CLI framework (clap) | ✅ Done |
| Configuration (TOML + XDG) | ✅ Done |
| Data models & errors | ✅ Done |
| SQLite database (backups, schedules, chunks, archive_chunks) | ✅ Done |
| Backup engine (bare git clone, streaming tar) | ✅ Done |
| Archive storage (zstd compression, HashingWriter) | ✅ Done |
| Restore engine (extract, ref checkout, dry-run) | ✅ Done |
| Content deduplication (ChunkStore, SHA-256, sharded) | ✅ Done |
| Theme engine (12 themes × 12 colors, live preview) | ✅ Done |
| Cron scheduler (crontab generation, validation) | ✅ Done |
| Spirit: forge word, forge reflect, forge rest | 🚧 In Progress |

### Phase 2 — Expansion

| Module | Status |
|--------|--------|
| AI agent harness (`forge breathe`) | 📋 Planned |
| Multi-agent orchestration (`forge strike`) | 📋 Planned |
| Credential vault | 📋 Planned |
| Prompt library | 📋 Planned |
| Project scaffolding | 📋 Planned |
| Scripture search & reference | 🚧 In Progress |
| Encrypted prayer journal | 🚧 In Progress |
| Sabbath mode | 🚧 In Progress |
| Dotfile management | 📋 Planned |
| System diagnostics | 📋 Planned |

### Phase 3 — Vision

| Module | Status |
|--------|--------|
| Creative tools (music, image, diagrams) | 📋 Planned |
| Notification hub | 📋 Planned |
| Webhook management | 📋 Planned |
| API gateway | 📋 Planned |
| Calendar integration | 📋 Planned |
| Cross-platform sync | 📋 Planned |

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
preferred = "opencode"
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

## Technical Decisions

1. **Everything is local first.** No cloud required. Network is optional enhancement.
2. **SQLite for all state.** No external databases. No daemons. Just files.
3. **Streaming pipelines.** No temp files. Pipe everything.
4. **Content-addressable storage.** Dedup at the chunk level across all projects.
5. **Bridge, don't compete.** Forge integrates with Hermes, Omarchy, llama-swap — it doesn't replace them.
6. **Encryption for private data.** Prayer journal, credentials — AES-256-GCM, local keys.
7. **Offline capable.** Scripture bundled. Backups local. Agents optional.

---

## The Creed

```
Forge is not a backup tool that grew too big.
Forge is a workshop that started with a solid anvil.

Every feature earns its place.
Every command maps to a concept you can remember.
Every pillar serves the smith, not the other way around.

The forge doesn't phone home.
The forge doesn't require the cloud.
The forge doesn't compete with its neighbors — it bridges to them.

We build in the open. We ship what works. We rest when it's time to rest.

"As iron sharpens iron, so one person sharpens another." — Proverbs 27:17
"For we are God's handiwork — His poem — created for good works." — Ephesians 2:10

Heat the coals. Strike the iron. Forge your future. 🔨
```

---

## The Vision

> *Genesis 4:22 — "Tubal-cain, who forged all kinds of tools out of bronze and iron."*

The first craftsman named in Scripture was a forger of tools. Not a king. Not a priest. A maker.

Forge exists because the terminal is the modern workshop, and the developer is the modern blacksmith. Every line of code is a hammer strike. Every git commit is a cooling blade. Every prayer is fuel for the fire.

We're building the tool that builds all other tools.

The forge doesn't ask permission. It doesn't wait for the cloud. It takes raw material — code, data, ideas, AI output, faith, creativity — and shapes it into something real. Something lasting. Something that outlives the smith.

> *"For we are God's handiwork, created in Christ Jesus to do good works, which God prepared in advance for us to do."* — Ephesians 2:10

The Greek word for "handiwork" is *poiēma*. It's where we get "poem."

You are a poem being written. Write good code. Do good works. Forge your future.

**This is the workshop. You are the smith.**

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. PRs welcome — every smith needs apprentices.

## License

Licensed under [Apache License 2.0](LICENSE).

---

*"The grid remembers everything. So should you."* 🎹🦞
