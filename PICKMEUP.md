# Forge + Forge Hub — Session Pickup

## Test Status (LAST VERIFIED: This Session)
- **Forge CLI (Rust):** 69 unit tests + 9 integration tests — **ALL GREEN** ✅
- **Forge Hub (Rails):** 85 examples, 0 failures — **ALL GREEN** ✅

---

## What's Complete

### Forge CLI (Rust) — `/home/synth/projects/forge`

| Phase | Module | Files | Status |
|-------|--------|-------|--------|
| Phase 1 | Core backup/restore/dedup/themes | `backup.rs`, `restore.rs`, `archive.rs`, `chunkstore.rs`, `db.rs`, `config.rs`, `models.rs`, `cli.rs`, `main.rs`, `theme.rs`, `theme_cmd.rs`, `scheduler.rs`, `error.rs` | ✅ Done |
| Phase 2A S6 | DRY utilities | `utils.rs` (shared `format_size` + `truncate_str`) | ✅ Done |
| Phase 2A S1 | Bible data layer | `spirit.rs` (21KB), `src/spirit/bible.db` (10MB, 66 books, 31,103 verses), `src/bin/generate_bible_db.rs` | ✅ Done |
| Phase 2A S3 | Encrypted journal | `reflect.rs` (19KB) — AES-256-GCM, key at `vault/journal.key`, `spirit.db` schema | ✅ Done |
| Phase 2A S7 | CLI wiring | `spirit_cmd.rs` (14KB) — themed handlers for `word`, `reflect`, `rest` commands | ✅ Done |
| Phase 2A | CLI dispatch | `cli.rs` updated with `Word(WordArgs)`, `Reflect(ReflectArgs)`, `Rest`. `main.rs` + `lib.rs` updated with mod declarations | ✅ Done |

**Cargo.toml dependencies added for Phase 2A:** `regex = "1"`, `aes-gcm = "0.10"`, `rand = "0.8"`

### Forge Hub (Rails) — `/home/synth/projects/forge-hub`

| Wave | Module | Files | Status |
|------|--------|-------|--------|
| Wave 1 | Dashboard + Anvil scaffold | Controllers, views, helpers, routes, Forge::Client, Forge::Database | ✅ Done |
| Wave 2 | T8-T12: Live data, backup list/detail, schedule viewer | Full request specs, pagination, real forge.db integration | ✅ Done |
| Wave 3 T13 | Backup trigger | `BackupJob` with SolidQueue + `Rails.cache` lock, trigger action, concurrent prevention | ✅ Done |
| Wave 3 T14 | Restore flow | `RestoreJob`, restore action with `turbo_confirm`, restore status display on detail page | ✅ Done |
| Wave 3 T15 | Turbo Streams | `BackupProgressChannel`, `backup_progress_controller.js` Stimulus, streaming via `Open3.popen3`, progress UI | ✅ Done |
| Wave 3 T16 | Schedule management | Schedules controller (create/destroy/toggle), views with forms, CLI integration | ✅ Done |
| — | Flash messages | Layout now renders notice/alert with synthwave styling | ✅ Done |
| — | Cache serialization | All cache values use string keys (`"status" => "success"`) instead of symbols | ✅ Done |

---

## What's Left to Do

### Immediate (Next Session Should Start Here)

#### 1. Fix Rust Audit CRITICALs (30 min)
**File: `/home/synth/projects/forge/src/db.rs`**
- `row_to_backup_entry` (line 58) uses positional column indices (`row.get(0)`, `row.get(1)`, etc.)
- **Fix to:** Named column access (`row.get("id")?`, `row.get("repo_name")?`, etc.)
- Same issue in `list_schedules` (line 308) — uses `row.get(0)` through `row.get(5)`
- This is fragile: if column order changes, it silently returns wrong data

**File: `/home/synth/projects/forge/src/backup.rs`**
- Temp dir (line 188-189) uses `forge-bare-{repo_name}-{pid}` — race condition if same process backs up same repo concurrently
- **Fix to:** Use `tempfile::tempdir()` for automatic cleanup and uniqueness (crate already in Cargo.toml)

#### 2. Rust Audit MEDIUM Fixes (Optional)
- `ForgeError` enum (`error.rs`) barely used vs `anyhow::Result` — consider consolidating
- `Forge::Database` (Ruby) opens new SQLite connection per query — flagged HIGH but not blocking
- `git2` IS used (backup.rs: branches, tags, stash, dirty check) — not dead weight

### Next Feature Work

#### Forge CLI — Phase 2B: Mind (AI Agent Routing)
Per `PLAN.md` Phase 2B:
1. `forge breathe` — Agent status dashboard (check Hermes, llama-swap, OpenCode)
2. `forge strike <task>` — Parse task, delegate to OpenCode
3. `forge breathe models` — Read llama-swap config, list available models
4. `forge breathe vault` — Credential management
5. `forge breathe prompts` — TOML-based prompt library CRUD

#### Forge Hub — Wave 4: Analytics
1. T17: Backup chart (Chart.js island — NOT React, use Stimulus)
2. T18: Dashboard statistics aggregation
3. T19: Archive contents browser

#### Forge Hub — Wave 5: Polish
1. T20-T24: All stub engines (Bellows, Flame, Tongs, Crucible, Bridge)
2. T25: Final navigation polish + custom error pages

### Eventually
- Git commit both projects
- Phase 2C: Incremental backups, verification, retention enforcement
- Phase 2D: System dashboard, theme builder/export

---

## Architecture Decisions (Don't Change These)

1. **No auth in v1** — localhost only
2. **No React** — Stimulus + ERB only
3. **Synthwave84 hardcoded CSS theme**
4. **Hybrid integration:** Forge Hub reads forge SQLite directly for queries, shells out to CLI for mutations
5. **Rust:** `anyhow::Result` everywhere, no `unwrap()`
6. **Bible data:** Pre-generated KJV SQLite at `src/spirit/bible.db` — zero network dependency
7. **Encrypted journal:** AES-256-GCM with random key at `~/.local/share/forge/vault/journal.key` (0600)
8. **Cache keys:** Always use string keys (`"status" => "success"`) — not symbols — for serialization safety

---

## Key File Map

### Forge CLI
```
/home/synth/projects/forge/
├── src/
│   ├── main.rs              — Entry point, dispatches CLI commands
│   ├── lib.rs               — Module declarations
│   ├── cli.rs               — clap CLI definitions (Word/Reflect/Rest added)
│   ├── config.rs            — TOML config, XDG dirs
│   ├── models.rs            — BackupEntry, ScheduleConfig, etc.
│   ├── db.rs                — SQLite CRUD (⚠️ positional indices → named columns)
│   ├── backup.rs            — Backup engine (⚠️ temp dir race)
│   ├── restore.rs           — Restore engine
│   ├── archive.rs           — tar + zstd pipeline
│   ├── chunkstore.rs        — Content-addressable storage
│   ├── scheduler.rs         — Cron scheduling
│   ├── theme.rs             — 12-theme color engine
│   ├── theme_cmd.rs         — Theme CLI commands
│   ├── utils.rs             — Shared format_size + truncate_str
│   ├── spirit.rs            — Bible data layer (Verse, daily_verse, search, lookup)
│   ├── reflect.rs           — Encrypted journal (AES-256-GCM)
│   ├── spirit_cmd.rs        — Word/Reflect/Rest CLI handlers
│   └── bin/
│       └── generate_bible_db.rs  — Bible DB generator
├── src/spirit/
│   └── bible.db             — Pre-built KJV SQLite (66 books, 31,103 verses)
├── PLAN.md                  — Master plan (all 6 pillars, phases)
└── Cargo.toml               — Deps: regex, aes-gcm, rand added
```

### Forge Hub
```
/home/synth/projects/forge-hub/
├── app/
│   ├── controllers/
│   │   ├── anvil/
│   │   │   ├── backups_controller.rb  — index, show, trigger, restore
│   │   │   └── schedules_controller.rb — index, create, destroy, toggle
│   │   ├── anvil_controller.rb        — Redirects to backups
│   │   └── dashboard_controller.rb    — Live stats from forge.db
│   ├── jobs/
│   │   ├── backup_job.rb              — Streaming + Rails.cache lock
│   │   └── restore_job.rb             — Background restore
│   ├── channels/
│   │   └── backup_progress_channel.rb — ActionCable
│   ├── javascript/controllers/
│   │   └── backup_progress_controller.js — Stimulus
│   ├── services/forge/
│   │   ├── client.rb                  — Shell out to forge CLI
│   │   └── database.rb               — Direct SQLite reads
│   ├── helpers/
│   │   └── anvil_helper.rb            — human_size, time_ago, etc.
│   └── views/
│       ├── layouts/
│       │   ├── application.html.erb   — ✅ Now renders flash messages
│       │   ├── _sidebar.html.erb
│       │   └── _topbar.html.erb
│       ├── anvil/
│       │   ├── backups/
│       │   │   ├── index.html.erb     — Backup list + trigger button + progress
│       │   │   └── show.html.erb      — Detail + restore button + status
│       │   ├── schedules/
│       │   │   └── index.html.erb     — Schedule list + add form
│       │   └── no_forge.html.erb      — Setup instructions
│       └── dashboard/
│           └── show.html.erb          — Live stats dashboard
├── config/
│   ├── routes.rb                      — All Wave 3 routes
│   └── initializers/forge.rb          — Method-style config (allows ENV override)
├── spec/
│   ├── requests/anvil/
│   │   ├── backups_spec.rb            — 17 tests
│   │   ├── backup_trigger_spec.rb     — 3 tests
│   │   ├── restore_spec.rb            — 7 tests
│   │   └── schedule_management_spec.rb — 6 tests
│   ├── requests/dashboard_spec.rb     — 4 tests
│   ├── requests/health_check_spec.rb  — 1 test
│   ├── requests/pillar_routes_spec.rb — 7 tests
│   ├── services/forge/
│   │   ├── client_spec.rb             — 13 tests
│   │   └── database_spec.rb           — 17 tests
│   ├── system/
│   │   ├── backup_progress_spec.rb    — 8 tests
│   │   └── sample_spec.rb             — 2 tests
│   └── rails_helper.rb
└── PICKMEUP.md                        — THIS FILE
```

---

## Bugs Fixed This Session

1. **Flash rendering missing from layout** — `<%= yield %>` had no flash message area. Added `notice`/`alert` rendering with synthwave-styled divs before yield.
2. **Cache symbol→string serialization** — `BackupJob` and `RestoreJob` wrote `:success`/`:error` symbols to Rails.cache. When read back, symbols became strings. Tests and views broke. Fixed everything to use string keys consistently: jobs → tests → views.
3. **`Forge::Config::DB_PATH` → `Forge::Config.db_path`** — Previous session fix in `no_forge.html.erb` (constant → method).

---

## How to Run

```bash
# Forge CLI (Rust)
cd /home/synth/projects/forge
cargo test                    # 69 unit + 9 integration tests
cargo build --release         # Binary at target/release/forge

# Forge Hub (Rails)
cd /home/synth/projects/forge-hub
bin/rspec                     # 85 examples
bin/rails server              # http://localhost:3000

# Key URLs
# http://localhost:3000                          → Dashboard
# http://localhost:3000/anvil/backups            → Backup list (102 backups)
# http://localhost:3000/anvil/backups/:id        → Backup detail + restore
# http://localhost:3000/anvil/schedules          → Schedule management
```

---

## Agent Sessions (for continuation if needed)
- Rust audit (explore): `ses_1bb57dd4bffe7Ueee4WZZCZ6np`
- Rails audit (explore): `ses_1bb57dd15ffed4ak4yMMqzAecH`
- S6 DRY utils (quick): `ses_1bb4ada79ffeYot61WVbn7At5P`
- S1 Bible data layer (deep): `ses_1bb4ada49ffeuJ8y0O3S9yHu9y`
- S3 Encrypted journal (deep): `ses_1bb4ada24ffeHgMBW45L0WexWJ`
- S7 CLI wiring (deep): `ses_1bb41f901ffe2jTfxJJ9iF2eKD`
- T13+T16 Backup trigger + schedules (deep): `ses_1bb4ad9ffffecbvfPtqL2SN5TI`
- T14 Restore flow (unspecified-high): `ses_1bb414c22ffeG0wIkOHPkZAj3n`
- T15 Turbo Streams (deep): `ses_1bb414bf2ffeZmQa3xeBRyLygE`

---

*Last updated: Session ending May 20, 2026*
*All tests green. Ready for next session.*
