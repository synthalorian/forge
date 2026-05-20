# FORGE — Development Plan

> *The master plan for building the tool that builds all other tools.*
> *Authored by synthclaw. Trusted to OpenCode + OMO.*

---

## Project State (Honest Assessment)

**Repository:** `/home/synth/projects/forge`
**Language:** Rust (edition 2021)
**Lines of code:** 2,490 (src/)
**Compiles:** Yes. Clean release build.
**Tests:** None. Dev dependencies declared but no test files exist.
**Git history:** Zero commits. Everything is untracked.
**Current capability:** Local git backup CLI with zstd compression, SHA-256 content deduplication, SQLite metadata, cron scheduling, and a 12-theme color engine.

### What Actually Works Right Now

These modules are fully implemented and functional:

| Module | File | Lines | What it does |
|--------|------|-------|--------------|
| CLI | `cli.rs` | 111 | clap-based argument parsing with 7 top-level commands |
| Config | `config.rs` | 93 | TOML load/save, XDG dirs, sensible defaults |
| Models | `models.rs` | 67 | BackupEntry, RepoSnapshot, ArchiveManifest, ChunkEntry, ScheduleConfig |
| Errors | `error.rs` | 33 | thiserror enums for all domain errors |
| Database | `db.rs` | 451 | SQLite schema (backups, schedules, chunks, archive_chunks), full CRUD, list/status queries |
| Backup Engine | `backup.rs` | 408 | Discovers repos, bare git clone, chunks through ChunkStore, progress bars, themed output |
| Archive Storage | `archive.rs` | 354 | Streaming tar → zstd pipeline, custom HashingWriter for SHA-256, verify mode |
| Restore Engine | `restore.rs` | 196 | Lookup backup, extract archive, optional ref checkout, dry-run |
| ChunkStore | `chunkstore.rs` | 81 | Content-addressable storage, zstd compression, hash-sharded paths |
| Scheduler | `scheduler.rs` | 180 | Cron expression validation, crontab file generation, add/remove/list |
| Theme Engine | `theme.rs` | 361 | 12 themes × 12 color slots, true color, style functions |
| Theme Commands | `theme_cmd.rs` | 87 | list, preview, set commands |

### What Needs Fixing Before Building New Features

1. **Remove tokio dependency** — It's declared in Cargo.toml with `features = ["full"]` but zero async code exists. Pure dead weight adding compile time and binary size.
2. **Verify git2 usage** — The backup engine shells out to `git clone --bare` via `Command`. If git2 (libgit2 binding) is unused, remove it. It's a heavy native dep.
3. **Make the first commit** — Project has zero git history. Everything is untracked. Commit the working v0.1 before touching anything.
4. **Write tests** — dev-deps are declared (tempfile, assert_cmd, predicates) but no tests exist. At minimum: backup → list → restore round-trip test.

---

## The Six Pillars (Feature Roadmap)

### I. Forge Code — The Anvil (Partially Built)

*Shape your projects with iron precision.*

**What exists:** Full git backup/restore engine with dedup, compression, scheduling.

**What to build:**

- [ ] Incremental backups — the ChunkStore foundation is ready. Need logic to compare current repo state against last backup's chunk manifest and only store new chunks.
- [ ] `forge anvil search <query>` — ripgrep across all backed-up repos
- [ ] `forge anvil audit` — dependency vulnerability check across registered projects
- [ ] `forge anvil health` — project status (uncommitted changes, stale branches, missing remotes)
- [ ] Multi-agent orchestration bridge (see Forge Mind — this overlaps)
- [ ] Project scaffolding templates (stretch goal, Phase 3)

**Recommendation:** Complete incremental backups first. It's the single highest-value technical feature. The chunk store is 80% there.

### II. Forge Mind — The Bellows (Not Started)

*Breathe life into your workflow with AI.*

**The approach:** Forge does NOT compete with Hermes. Forge bridges to it.

```
forge strike <task>
  ↓
Forge reads task, determines agent requirements
  ↓
Routes to Hermes API / delegates to OpenCode / calls Codex CLI
  ↓
Captures output, stores in session history
  ↓
Returns result with themed output
```

**What to build:**

- [ ] `forge breathe` — Agent status dashboard (check what's running: Hermes, llama-swap, OpenCode)
- [ ] `forge strike <task>` — Parse task, route to best available agent
- [ ] `forge breathe models` — List models from local llama-swap config + cloud providers
- [ ] `forge breathe vault` — Read/write credentials (bridge to Hermes auth.json format)
- [ ] `forge breathe prompts` — CRUD for a local prompt library (TOML files in ~/.forge/prompts/)
- [ ] `forge breathe pipe` — Define multi-step agent pipelines in TOML
- [ ] Session persistence — Store agent conversation state in SQLite

**Agent priority order for routing:**
1. OpenCode (local, via OMO, Z.AI endpoint)
2. llama-swap (local, GPU-accelerated, offline)
3. Hermes (remote, full toolset)
4. Codex CLI (if available)

**Key integration points:**
- Hermes: `~/.hermes/auth.json` for credentials, `hermes` CLI for agent tasks
- llama-swap: Read `/home/synth/llama.cpp/llama-swap/config.yaml` for model inventory
- OpenCode: `opencode` CLI with oh-my-openagent (OMO v4.2.2)
- Z.AI: Endpoint `https://api.z.ai/api/coding/paas/v4`, key in auth.json

**Recommendation:** Start with `forge breathe` (status dashboard) and `forge strike` (basic task routing to OpenCode). Get the harness working with one agent before adding more.

### III. Forge Spirit — The Flame (Not Started)

*The fire that tempers the steel.*

This is Forge's soul. Nobody else is building this. It's the differentiator.

**What to build:**

- [ ] Bundle a Bible JSON/SQLite dataset locally (public domain translation — WEB, KJV, or ESV via API with offline cache)
- [ ] `forge word` — Display today's verse (date-seeded selection)
- [ ] `forge word search <query>` — Full-text verse search
- [ ] `forge word <reference>` — Look up specific passage (e.g., `forge word proverbs 27:17`)
- [ ] `forge reflect` — Open prayer journal (encrypted SQLite, AES-256)
- [ ] `forge reflect entry <text>` — Write a journal entry
- [ ] `forge reflect history` — Browse past entries
- [ ] `forge rest` — Sabbath mode: stop all forge processes, kill background agents, display a verse about rest
- [ ] Reflection prompts — Tied to work patterns (e.g., after X hours of coding, prompt a reflection)

**Data source options:**
1. Bundle KJV (public domain, no license issues) as SQLite — ~3MB compressed
2. Use bible-api.com for ESV/NIV with offline cache
3. Ship with WEB (World English Bible) — public domain, modern English

**Recommendation:** Bundle KJV as a SQLite database. Zero network dependency. Zero license issues. Add API-based translations as optional feature. Build this module early — it's small (maybe 400 lines) and it gives Forge an identity nothing else has.

### IV. Forge System — The Tongs (Partially Built)

*Grip and shape your environment.*

**What exists:** Theme engine with 12 themes, config management, XDG-compliant paths.

**What to build:**

- [ ] `forge grip` — System dashboard (CPU, memory, disk, GPU if available, running services)
- [ ] `forge grip dotfiles` — Track and version dotfiles with git
- [ ] `forge grip dotfiles restore` — Restore dotfiles from any point in history
- [ ] `forge grip services` — List running dev services (rails servers, llama-swap, etc.)
- [ ] `forge grip diagnose` — System health check (inspired by `omarchy debug`)
- [ ] `forge theme create` — Interactive theme builder
- [ ] `forge theme export` — Export theme for use in other apps (Alacritty, Kitty, Ghostty)

**Omarchy integration notes:**
- Omarchy manages `~/.config/` — Forge should NOT conflict with it
- Forge can READ Omarchy state (theme, config) but should write only to `~/.forge/`
- Safe to wrap `omarchy` CLI commands for display in Forge dashboard

**Recommendation:** Start with `forge grip` (system dashboard) and `forge theme create`. The theme engine is already built — making it extensible is low effort, high value.

### V. Forge Create — The Crucible (Not Started)

*Where raw material becomes something beautiful.*

**What to build:**

- [ ] `forge melt chords <key> <scale>` — Generate chord progressions
- [ ] `forge melt chords suggest <mood>` — Suggest progressions by mood/style
- [ ] `forge melt palette` — Generate color palettes (complementary, analogous, triadic)
- [ ] `forge melt palette from-image <path>` — Extract palette from an image
- [ ] `forge melt diagram <type>` — Generate ASCII/SVG diagrams (flowchart, sequence, architecture)
- [ ] `forge melt image <prompt>` — Bridge to image generation (ComfyUI or Hermes)
- [ ] `forge melt markdown <file>` — Markdown preview/render

**Recommendation:** Phase 3. Music theory helpers first (small, self-contained, no external deps). Image gen can bridge to existing tools.

### VI. Forge Connect — The Bridge (Not Started)

*Link everything together.*

**What to build:**

- [ ] `forge bridge` — Show connection status for all integrations
- [ ] `forge bridge hooks` — Manage webhook endpoints
- [ ] `forge bridge notify` — Send test notifications (desktop, Telegram, Discord)
- [ ] `forge bridge sync` — Sync task state across platforms
- [ ] Notification routing — Intercept Forge events and push to configured channels

**Recommendation:** Phase 3. This is glue code that becomes useful once the other pillars have events worth routing.

---

## Development Phases

### Phase 1: Solidify the Foundation

**Goal:** Ship v0.1.0 as a real, tested, committed open-source project.

1. `git init && git add . && git commit -m "v0.1: Forge foundation — backup, restore, dedup, themes"`
2. Remove tokio from Cargo.toml (unused)
3. Verify and potentially remove git2 (if backup only uses `Command`)
4. Write tests:
   - `forge init` → config file created
   - `forge backup .` → archive created, DB entry exists
   - `forge list` → shows the backup
   - `forge restore <id>` → files extracted correctly
   - Theme round-trip: set → load → verify
5. Create `.github/workflows/ci.yml` with `cargo test`, `cargo clippy`, `cargo fmt --check`
6. Create GitHub repo `synthalorian/forge`, push

### Phase 2A: Spirit — Give It a Soul

**Goal:** Forge becomes the only dev tool with integrated faith practice.

1. Bundle KJV as SQLite (`src/spirit/bible.db` or generated at build time)
2. Build `forge word` commands (daily verse, search, reference lookup)
3. Build `forge reflect` commands (encrypted journal with AES-256)
4. Build `forge rest` (Sabbath mode — kill processes, show rest verse)
5. Add `[spirit]` section to config.toml
6. Write tests for all spirit commands

### Phase 2B: Mind — Light the Fire

**Goal:** Forge can route tasks to AI agents.

1. Build `forge breathe` (agent status — check for running: Hermes, llama-swap, OpenCode)
2. Build `forge strike <task>` (parse task, delegate to OpenCode as first agent)
3. Build `forge breathe models` (read llama-swap config, list available)
4. Build `forge breathe vault` (credential management, bridge to Hermes auth format)
5. Build `forge breathe prompts` (TOML-based prompt library CRUD)
6. Add session persistence to agents.db
7. Write tests

### Phase 2C: Code — Sharpen the Blade

**Goal:** The backup engine becomes best-in-class.

1. Implement incremental backups using existing ChunkStore
2. Build `forge temper` (backup verification — re-hash and compare)
3. Implement retention policy enforcement (auto-prune old backups)
4. Build `forge anvil search` (cross-repo code search)
5. Build `forge anvil health` (project status dashboard)
6. Add backup size trending to `forge status`
7. Write tests

### Phase 2D: System — Grip Everything

**Goal:** Forge manages your environment.

1. Build `forge grip` (system resource dashboard)
2. Build `forge theme create` (interactive theme builder)
3. Build `forge theme export` (export to terminal config formats)
4. Build `forge grip dotfiles` (track and version)
5. Build `forge grip diagnose` (system health check)
6. Write tests

### Phase 3: Expand the Workshop

**Goal:** Creative tools and connections.

1. Forge Create: music theory helpers, palette generator, diagram generator
2. Forge Connect: webhook management, notification hub, task sync
3. Project scaffolding templates
4. Plugin/extension system (stretch goal)

---

## Technical Decisions

### Architecture Principles

1. **Everything is local first.** No cloud required. Network is optional enhancement.
2. **SQLite for all state.** No external databases. No daemons. Just files.
3. **Streaming pipelines.** No temp files. Pipe everything. (Already done in archive.rs)
4. **Content-addressable storage.** Dedup at the chunk level. (Already done in chunkstore.rs)
5. **Bridge, don't compete.** Forge integrates with Hermes, Omarchy, llama-swap — it doesn't replace them.
6. **Encryption for private data.** Prayer journal, credentials — AES-256, local keys.
7. **Offline capable.** Scripture bundled. Backups local. Agents optional.

### Directory Layout

```
~/.forge/
├── config.toml            Main configuration
├── vault/                 Encrypted credentials (AES-256-GCM)
│   └── credentials.enc
├── db/
│   ├── forge.db           Backups, projects, schedules
│   ├── spirit.db          Journal + scripture bookmarks
│   └── agents.db          Agent sessions, pipeline state
├── archives/              Compressed git backups (.tar.zst)
├── chunks/                Content-dedup store (SHA-256 sharded)
├── projects/              Project registry metadata
├── prompts/               Prompt library (TOML files)
├── themes/                Custom user themes (TOML)
├── scripts/               Lifecycle hooks (pre-backup, post-strike, etc.)
├── logs/                  Activity history
└── data/
    └── bible.db           Bundled scripture (KJV)
```

### New Dependencies to Add

| Crate | Purpose | Phase |
|-------|---------|-------|
| `aes-gcm` | Encrypted journal and vault | 2A |
| `rand` | Cryptographic key generation | 2A |
| `serde_json` | Already included — Bible data parsing | 2A |
| `sysinfo` | System resource monitoring | 2D |
| `regex` | Verse reference parsing | 2A |
| `keyring` | OS keychain integration (optional) | 2B |

### Dependencies to Remove

| Crate | Reason |
|-------|--------|
| `tokio` | No async code exists. Remove entirely or commit to async. |

### Dependencies to Verify

| Crate | Question |
|-------|----------|
| `git2` | Backup shells out to `git`. Is git2 used anywhere? If not, remove — it's a heavy native dep. |

---

## OpenCode + OMO Integration

This project is entrusted to OpenCode with oh-my-openagent for ongoing development.

**How to work on Forge with OpenCode:**

```bash
# From the project directory
cd /home/synth/projects/forge
opencode
```

**OMO agent configuration:**
- Default model: glm-5.1 (Z.AI endpoint)
- OMO version: v4.2.2
- The agent has full context from this plan document

**What OpenCode should focus on:**
1. Follow the phases in order — don't skip ahead
2. Every new module gets tests before being considered "done"
3. Keep the blacksmith metaphor alive in code comments and CLI output
4. All new commands use the theme engine for output (see `theme.rs`)
5. New modules follow the existing pattern: `modulename.rs` + `modulename_cmd.rs` if it has subcommands
6. Config additions go in `config.toml` with sensible defaults that don't break existing configs

**What synthclaw does:**
- Architectural guidance and design decisions
- Code review when synth requests it
- Roadmap updates as the project evolves
- Keeping the vision coherent across all six pillars

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

"As iron sharpens iron, so one person sharpens another."
"For we are God's handiwork — His poem — created for good works."

Heat the coals. Strike the iron. Forge your future. 🔨
```

---

*This document is the source of truth for Forge development.*
*Last updated: May 2026*
*Maintained by: synthclaw | Development led by: OpenCode + OMO*
*Owner: synth (synthalorian)*
