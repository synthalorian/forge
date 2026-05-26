<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/forge-icon.png">
  <img src="assets/forge-icon.png" alt="Forge" width="400">
</picture>

# Forge

**Craft Your Digital Future.**

The blacksmith was the cornerstone of every civilization. They built the tools that built everything else. The sword, the plow, the cathedral — all born at the forge.

Forge is that for the digital age.

A CLI platform and web dashboard where human creativity meets artificial intelligence. Where you don't just store code — you craft your entire digital life. Git backups. AI orchestration. Scripture. System management. Creative tools. All connected. All local. All yours.

**One forge to shape them.**

---

## 📦 What's Inside

| Component | Description | Language | Status |
|-----------|-------------|----------|--------|
| **Forge CLI** | Terminal workshop — backup, restore, scripture, themes, agents | Rust | ✅ Active |
| **Forge Hub** | Visual command center — web GUI with synthwave84 theme | Ruby on Rails 8 | ✅ Active |

```
forge/
├── src/              → Forge CLI (Rust binary)
├── hub/              → Forge Hub (Rails 8 web app)
├── assets/           → App icon, branding
├── Cargo.toml        → CLI build config
└── hub/Gemfile       → Hub dependencies
```

---

## 🔨 Forge CLI

The terminal workshop. Everything starts here.

### The Six Pillars

| Pillar | Command | Purpose | Status |
|--------|---------|---------|--------|
| **Anvil** | `forge anvil` | Backup & restore management | ✅ Done |
| **Bellows** | `forge breathe` | AI agent orchestration | ✅ Done |
| **Flame** | `forge word` | Scripture & study | ✅ Done |
| **Tongs** | `forge grip` | System management | ✅ Done |
| **Crucible** | `forge melt` | Creative tools | ✅ Done |
| **Bridge** | `forge bridge` | Connections & integrations | ✅ Done |

### Install from Source

**Prerequisites:** Rust 1.75+ (stable), Git, a C compiler (for libgit2, zstd)

```bash
git clone https://github.com/synthalorian/forge.git
cd forge
cargo install --path .
```

### Quick Start

```bash
# Initialize
forge init

# Backup a project
forge quench /path/to/project

# See your backups
forge list

# Set the theme
forge theme set synthwave84
```

### Command Reference

```
forge                              Dashboard — what's happening right now
forge init                         First-time setup
forge status                       System & backup health overview

# The Workshop Verbs
forge quench [path]                Backup git repos
forge restore <id>                 Restore from backup
forge heat                         Spin up AI agents
forge strike <task>                Execute task via best available agent
forge temper                       Review recent work, suggest improvements
forge anneal                       Enter deep work mode (do not disturb)
forge alloy <sources>              Merge outputs from multiple agents
forge cast                         Deploy current project
forge grind                        Run linters, tests, quality checks
forge polish                       Format and document

# Scripture & Faith
forge word                         Daily scripture
forge word search <query>          Search verses
forge word ref <passage>           Look up a passage
forge reflect                      Prayer journal (AES-256-GCM encrypted)
forge rest                         Sabbath mode — shut it all down

# System & Creative
forge grip                         System dashboard (CPU, memory, disk, GPU)
forge melt chords                  Chord progressions and music theory
forge melt palette                 Color palette generation
forge melt diagram                 ASCII and SVG architecture diagrams

# Personalization
forge theme list                   Browse 12 built-in themes
forge theme preview <name>         See a theme in action
forge theme set <name>             Apply a theme
forge theme create                 Build your own theme (TOML)
forge theme export <name>          Export theme to Alacritty/Kitty/Ghostty

# Agent Pipelines
forge breathe pipe <file>          Run multi-step agent pipeline from TOML definition
```

### Built-in Themes

12 themes, each with 12 color slots. Built for terminals that speak true color.

| Theme | Description |
|-------|-------------|
| `synthwave84` | Neon purple on deep black — the default 🔮 |
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

### Architecture

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

The ChunkStore splits data into 4MB blocks, SHA-256 hashes each one, compresses with zstd, and stores them in a sharded content-addressable layout (`chunks/ab/cdef...zst`). New backups only store chunks they haven't seen before. Across all your projects, identical dependencies, assets, and boilerplate are stored once.

---

## 🖥️ Forge Hub

The visual command center. A Rails 8 web GUI that sits on top of Forge CLI, giving you a synthwave-styled dashboard for your forge infrastructure.

### Features

- **Dashboard** — At-a-glance stats: backup count, repo count, storage used, active schedules, weekly trends
- **Backup Browser** — Browse all backups with search, pagination, restore, and chart data
- **Schedule Manager** — Create, toggle, and delete backup schedules with cron expressions
- **Flame** — Scripture search (debounced live search), reference lookup, encrypted journal browser with pagination
- **Bellows** — Agent detection, session management, chat-style message history, quick strike, pipeline runner
- **Tongs** — System dashboard with GPU/temperatures/resource bars, diagnostics, dotfiles tracker, services list
- **Crucible** — Creative tools bridge (chords, palettes, diagrams, markdown, image generation, palette from-image extraction)
- **Bridge** — Integration status for 11+ tools, lifecycle hooks, sync dashboard, notification testing, Omarchy detection
- **Synthwave84 Theme** — Deep purple palette with neon accents, CRT scanlines, horizon glow, glass morphism
- **Theme Switcher** — Toggle between Synthwave84, Midnight, Ocean, and Light variants
- **Global Search** — Search across all pillars from a unified search bar

### Install & Run

**Prerequisites:** Ruby 3.2+, Bundler, Rails 8.1+, Forge CLI installed and initialized

```bash
# From the repo root
cd hub

# Install dependencies
bundle install

# Set up the database
bin/rails db:create db:migrate

# Build Tailwind CSS
bin/rails tailwindcss:build

# Start the server
bin/rails server

# Open in browser
# → http://localhost:3000
```

### Hub Configuration

The Hub reads from the same `~/.forge/` directory as the CLI. No separate configuration needed — it shares the SQLite databases, archive store, and theme settings.

### Development

```bash
# Run with hot-reload (Tailwind + Stimulus)
bin/dev

# Run tests
bin/rails test

# Rebuild Tailwind after theme changes
bin/rails tailwindcss:build
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | Ruby on Rails 8.1 |
| Frontend | Tailwind CSS v4 + Stimulus.js |
| Database | SQLite (shared with CLI) |
| Asset Pipeline | Propshaft |
| Web Server | Puma |
| Theme | Synthwave84 (Omarchy-aligned purple palette) |

---

## ⚙️ Configuration

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

## 🚀 Releases

Each release ships:

| Asset | Description |
|-------|-------------|
| `forge` | Linux x86_64 binary (statically linked) |
| `forge-hub.tar.gz` | Rails app bundle (hub/ directory) |
| `forge-icon.png` | App icon for desktop integrations |

Download the latest from the [Releases page](https://github.com/synthalorian/forge/releases).

```bash
# Install CLI from release binary
curl -sL https://github.com/synthalorian/forge/releases/latest/download/forge -o ~/.local/bin/forge
chmod +x ~/.local/bin/forge
forge init
```

---

## 🗺️ Roadmap

### Phase 1 — Foundation ✅

| Module | Status |
|--------|--------|
| CLI framework (clap) | ✅ Done |
| Configuration (TOML + XDG) | ✅ Done |
| Data models & errors | ✅ Done |
| SQLite database | ✅ Done |
| Backup engine (bare git clone, streaming tar) | ✅ Done |
| Archive storage (zstd compression) | ✅ Done |
| Restore engine (extract, ref checkout, dry-run) | ✅ Done |
| Content deduplication (ChunkStore, SHA-256) | ✅ Done |
| Theme engine (12 themes × 12 colors) | ✅ Done |
| Cron scheduler | ✅ Done |
| Forge Hub — Rails 8 web GUI | ✅ Done |

### Phase 2 — Expansion ✅

| Module | Status |
|--------|--------|
| Scripture search & reference | ✅ Done |
| Encrypted prayer journal | ✅ Done |
| Sabbath mode | ✅ Done |
| AI agent harness (`forge breathe`) | ✅ Done |
| Multi-agent orchestration (`forge strike`) | ✅ Done |
| Credential vault | ✅ Done |
| Prompt library | ✅ Done |
| System diagnostics | ✅ Done |
| Hub — real-time backup progress | ✅ Done |
| Hub — agent status dashboard | ✅ Done |

### Phase 3 — Creative & Integration ✅

| Module | Status |
|--------|--------|
| Creative tools (music, image, diagrams) | ✅ Done |
| Notification hub | ✅ Done |
| Webhook management | ✅ Done |
| Session persistence | ✅ Done |
| Markdown renderer | ✅ Done |
| Procedural image generation | ✅ Done |
| Bridge sync | ✅ Done |
| Hub — full pillar pages (all 6) | ✅ Done |
| Hub — journal browser with pagination | ✅ Done |
| Hub — session detail with message history | ✅ Done |
| Hub — dotfiles management | ✅ Done |
| Hub — global search | ✅ Done |
||
| **v1.1.0 — Hub Polish** | |
| Hub — per-tab Turbo Stream output for Crucible (chords/palette/diagram) | ✅ Done |
| Hub — palette from-image extraction via CLI bridge | ✅ Done |
| Hub — Stimulus dotfile tracker (replaces inline JS) | ✅ Done |
| Hub — CLI/UI feature parity alignment (harmonies, diagram types) | ✅ Done |

---

## Technical Decisions

1. **Everything is local first.** No cloud required. Network is optional enhancement.
2. **SQLite for all state.** No external databases. No daemons. Just files.
3. **Streaming pipelines.** No temp files. Pipe everything.
4. **Content-addressable storage.** Dedup at the chunk level across all projects.
5. **Bridge, don't compete.** Forge integrates with Hermes, Omarchy, llama-swap — it doesn't replace them.
6. **Encryption for private data.** Prayer journal, credentials — AES-256-GCM, local keys.
7. **Offline capable.** Scripture bundled. Backups local. Agents optional.
8. **One repo, two surfaces.** CLI for speed. Hub for visibility. Same data, same forge.

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
