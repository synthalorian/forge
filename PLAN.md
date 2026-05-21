# FORGE — Development Plan

> *The master plan for building the tool that builds all other tools.*
> *Last updated: May 20, 2026*

---

## Project State (Current)

**Repository:** `synthalorian/forge` (public, monorepo)
**Language:** Rust (edition 2021) + Ruby on Rails 8.1 (Hub)
**CLI Binary:** `forge` — 23 commands across 6 pillars + workshop verbs
**Hub:** Rails 8.1 web GUI with full pillar pages, session management, journal browser, dotfiles, global search
**Lines of code:** ~12,000 (8,500 Rust src/ + ~3,500 Ruby/ERB/JS)
**Tests:** 199 CLI (all green) + 224 Hub specs (all green)
**GitHub Actions:** CI (cargo test + clippy) + Release (tag-triggered binary upload)
**Latest release:** v1.0.0
**Data:** 102 backups across 99 git repos, 2.0 GB stored

### What Works Right Now

Every module across all six pillars is fully implemented and tested:

| Module | File | Lines | Features |
|--------|------|-------|----------|
| CLI | `src/cli.rs` | 347 | 23 top-level clap commands with aliases |
| Config | `src/config.rs` | 93 | TOML load/save, XDG dirs, retention policy |
| Models | `src/models.rs` | 67 | BackupEntry, RepoSnapshot, ChunkEntry, etc. |
| Errors | `src/error.rs` | 33 | thiserror ForgeError enum |
| Database | `src/db.rs` | 451 | SQLite schema (backups, schedules, chunks), CRUD with indexes |
| Backup | `src/backup.rs` | 390 | Repo discovery, bare git clone, incremental support |
| Archive | `src/archive.rs` | 520 | Tar→zstd pipeline, HashingWriter, content-addressable chunks |
| Restore | `src/restore.rs` | 196 | DB lookup, extract, ref checkout, dry-run |
| Scheduler | `src/scheduler.rs` | 180 | Cron validation, crontab files, add/remove/list |
| ChunkStore | `src/chunkstore.rs` | 81 | 4MB chunks, SHA-256, zstd, sharded paths |
| Theme Engine | `src/theme.rs` | 690 | 12 built-in themes + custom TOML themes, raw 24-bit ANSI |
| Theme Cmd | `src/theme_cmd.rs` | 240+ | list, preview, set, create (interactive builder) |
| Anvil | `src/anvil.rs` | 727 | temper (verify), prune (retention), search, health |
| Bridge | `src/bridge.rs` | 275 | status, hooks, notify (desktop/telegram/discord) |
| Crucible | `src/crucible.rs` | 613 | chords, palette, diagram (music theory + colors + ASCII) |
| Mind | `src/mind.rs` | 544 | Agent detection (OpenCode, llama-swap, Hermes, Codex), routing |
| Mind Cmd | `src/mind_cmd.rs` | 278 | breathe (status/models/vault/prompts), strike |
| Spirit | `src/spirit.rs` | 718 | Bible DB (KJV), daily verse, search, reference, abbreviations |
| Spirit Cmd | `src/spirit_cmd.rs` | 412 | word, reflect, rest CLI dispatch |
| Reflect | `src/reflect.rs` | 610 | AES-256-GCM encrypted journal, CRUD, search, pagination |
| Tongs | `src/tongs.rs` | 592 | grip dashboard (CPU/memory/disk/GPU), dotfiles, diagnose, services |
| Workshop | `src/workshop.rs` | 824 | heat, anneal, alloy, cast, grind, polish |
| Bible DB Gen | `src/bin/generate_bible_db.rs` | ~100 | Build-time KJV SQLite database generator |

### Hub — Rails 8.1 (hub/)

| Page | Controller | View | Status |
|------|-----------|------|--------|
| Dashboard | `dashboard_controller.rb` | `dashboard/show.html.erb` | ✅ Live stats, top repos, latest backup, pillar cards |
| Anvil | `anvil_controller.rb` + backup/schedules CRUD | Full set of views | ✅ Filters, search, browse, chart data |
| Bellows | `bellows_controller.rb` | `bellows/index.html.erb` | ✅ Agent detection with status dots |
| Flame | `flame_controller.rb` | `flame/index.html.erb` | ✅ Daily verse, journal count, commands |
| Tongs | `tongs_controller.rb` | `tongs/index.html.erb` | ✅ System info, resource usage, commands |
| Crucible | `crucible_controller.rb` | `crucible/index.html.erb` | ✅ Three-tab interactive forge melt bridge |
| Bridge | `bridge_controller.rb` | `bridge/index.html.erb` | ✅ Integration status, hooks, commands |

---

## What's Left (Minor Polish)

The project is feature-complete. Remaining items are polish and release management:

### v0.3.0 Milestone ✅

- [x] Tag and cut v0.3.0 release on GitHub
- [x] Add CLI aliases: `forge q` → quench, `forge s` → strike, `forge l` → list
- [x] `forge theme export` — Export custom theme to Alacritty/Kitty/Ghostty TOML format
- [x] Show backup type (Full/Incremental) in `forge status` and `forge list` output
- [x] Add `forge breathe pipe` — Multi-step agent pipeline definitions in TOML

### v0.4.0 Milestone ✅

- [x] `forge grip diagnose` — Extended system health with GPU detection, Omarchy checks, temperature, network, disk health, and fix suggestions
- [x] `forge melt palette from-image` — Extract dominant color palette from an image file (`forge melt palette --file photo.png`)

### v0.5.0 Milestone ✅

- [x] `forge melt markdown` — Render markdown files to terminal with formatting, colors, and headers

### v0.6.0 Milestone ✅

- [x] `forge melt image` — Procedural image generation from text prompts (abstract art, gradients, color fields) using the image crate

### v0.7.0 Milestone ✅

- [x] Session persistence — Store agent conversation state in agents.db for continuity across restarts

### v0.8.0 Milestone ✅

- [x] `forge bridge sync` — Sync task state across platforms (Hermes, local tasks, schedules)

### v0.9.0 Milestone ✅

- [x] Dashboard: Remove "Coming Soon" from all pillar cards — all 6 pillars now active
- [x] Hub — Flame: Interactive scripture search, reference lookup, journal browsing
- [x] Hub — Bellows: Agent session management, quick strike, pipeline runner
- [x] Hub — Tongs: Diagnose output, GPU info, services list, dotfiles tracker
- [x] Hub — Bridge: Sync dashboard, test notification form, Omarchy detection
- [x] Hub — Specs: Request specs for all new pillar actions (49 new specs)
- [x] Version bump to v0.9.0 (Cargo.toml + sidebar)
- [x] Open3 timeout fixes on all controllers that shell out to forge CLI

### v0.10.0 Milestone 🚧

- [ ] Hub — Flame: Stimulus debounced live scripture search
- [ ] Hub — Flame: Journal entries browser with pagination
- [ ] Hub — Bellows: Agent session detail page with message history
- [ ] Hub — Tongs: Dotfiles management (track/list/restore via forge grip dotfiles)

### v0.10.0 Milestone ✅

- [x] Hub — Flame: Stimulus debounced live scripture search
- [x] Hub — Flame: Journal entries browser with pagination + search
- [x] Hub — Bellows: Agent session detail page with chat-like message history
- [x] Hub — Tongs: Dotfiles management (track/list/restore via forge grip dotfiles)

### v0.11.0 Milestone 🚧

- [ ] Hub — Global search across all pillars (agent in progress)

### v1.0.0 Milestone ✅

- [x] Version bump to 1.0.0 (Cargo.toml + sidebar)
- [x] All 6 pillars fully interactive in both CLI and Hub
- [x] Dashboard shows all pillars as active
- [x] Hub: 224+ specs passing, CLI: 199 tests passing
- [x] Open3 timeout protection on all CLI-bridging controllers
- [x] Stimulus debounced scripture search
- [x] Journal browser with pagination
- [x] Agent session management with chat-style message history
- [x] Dotfiles management (track/list/restore)
- [x] System diagnostics with GPU, temperatures, services
- [x] Bridge sync dashboard with notification testing
- [x] Omarchy integration detection

---

## Architecture

```
~/.forge/
├── config.toml            Main configuration (TOML)
├── vault/                 Encrypted credentials (AES-256-GCM)
├── db/
│   ├── forge.db           Backups, schedules, chunks
│   └── spirit.db          Journal entries (encrypted)
├── archives/              Compressed git backups (.tar.zst + .manifest.json)
├── chunks/                Content-addressable store (sha256/<2-hex>/<rest>.zst)
├── themes/                Custom user themes (.toml)
├── scripts/               Lifecycle hooks (pre/post-backup, post-strike)
└── logs/                  Activity history
```

### Key Design Decisions

1. **Local first.** No cloud required. Network is optional enhancement.
2. **SQLite for all state.** No external databases. No daemons. Just files.
3. **Streaming pipelines.** No temp files. Pipe everything (tar | zstd | hasher).
4. **Content-addressable chunks.** Dedup across all projects at the 4MB block level.
5. **Bridge, don't compete.** Integrates with Hermes, Omarchy, llama-swap — doesn't replace them.
6. **Encryption at rest.** Prayer journal uses AES-256-GCM, local keys.
7. **Offline capable.** Scripture bundled (KJV SQLite). Backups local. Agents optional.
8. **Raw 24-bit ANSI.** No `colored` crate — custom StyledString emits exact truecolor codes.
9. **One monorepo, two surfaces.** CLI for speed. Hub for visibility. Same data, same forge.

---

## Technical Details

### Dependencies

```toml
clap = "4"           # CLI (derive)
git2 = "0.20"        # Git metadata (branches, tags, stashes, commits)
rusqlite = "0.33"    # SQLite (bundled)
zstd = "0.13"        # Compression
sha2 = "0.10"        # Chunk hashing
serde = "1"          # Serialization (derive)
toml = "0.8"         # Config + custom themes
chrono = "0.4"       # Timestamps
indicatif = "0.17"   # Progress bars
aes-gcm = "0.10"     # Encrypted journal
regex = "1"          # Verse reference parsing
walkdir = "2"        # Filesystem traversal
anyhow/thiserror     # Error handling
```

**Explicitly removed:** `tokio` (no async code), `colored` (silent truecolor downgrade)

### CI/CD

- **CI (push/PR to main):** `cargo check` → `cargo fmt --check` → `cargo clippy -D warnings` → `cargo test`
- **Release (push v* tag):** Builds release binary → strips → packages hub tarball → uploads `forge`, `forge-hub.tar.gz`, `forge-icon.png` + SHA256 checksums → auto-generates release notes

---

## Session Handoff (Pickup Instructions)

When resuming work on Forge, follow this checklist:

### 1. Check current state
```bash
cd ~/projects/forge
git log --oneline -5          # Last 5 commits
git status --short            # Any unstaged/untracked work
gh release view --json assets | head -20   # Release assets
cargo test 2>&1 | tail -5     # Tests green?
```

### 2. Common starting points
- **Cut a new release:** `git tag v0.3.0 && git push origin v0.3.0` — triggers release.yml workflow
- **Add CLI alias:** In `src/cli.rs`, add `alias = "x"` to the `#[command()]` attribute on the enum variant
- **Add Hub pillar feature:** Create controller action → wire route → create view → add Stimulus controller if needed

### 3. Build commands
```bash
cargo build --release         # CLI binary
cp target/release/forge ~/.local/bin/forge   # Install locally

# Hub (from hub/ directory)
cd hub
bin/rails tailwindcss:build   # Rebuild CSS
bin/rails server -p 3003      # Start server
```

### 4. Key files
- `src/cli.rs` — All command definitions
- `src/main.rs` — Command dispatch
- `src/theme.rs` — Theme engine + custom theme loading
- `hub/config/routes.rb` — All routes
- `hub/app/controllers/*` — Controllers
- `.github/workflows/` — CI + Release workflows

---

*"The grid remembers everything. So should you."* 🎹🦞
