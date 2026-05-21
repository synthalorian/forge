# Forge Code Review — Surgical Audit

**Date**: 2026-05-21
**Reviewer**: Hermes Agent (automated)
**Scope**: Rust CLI `src/` + Rails 8 Hub `hub/`
**Author of code**: GLM-5.1 via OpenCode/Omo agents

---

## CRITICAL

### C1. Command Injection via `safe_command()` in Rust `tongs.rs`
- **File**: `src/tongs.rs:774-780` and all 31 call sites
- **Category**: security
- **Description**: `safe_command()` passes arbitrary strings to `sh -c`. Two call sites in `bridge.rs:391-396` use `format!()` to interpolate `cfg.archive_dir` (a user-controlled path) directly into a shell command string:
  ```rust
  let forge_size = crate::tongs::safe_command(&format!(
      "du -sh {} 2>/dev/null | cut -f1",
      cfg.archive_dir.parent().unwrap_or(&cfg.archive_dir).display()
  ));
  ```
  If `archive_dir` contains shell metacharacters (spaces, `$()`, backticks, `;`), this enables command injection.
- **Fix**: Use `std::process::Command` with explicit argument arrays instead of shell execution, or at minimum shell-escape the path.

### C2. Command Injection in Rails `TongsController#safe_command`
- **File**: `hub/app/controllers/tongs_controller.rb:76`
- **Category**: security
- **Description**: `` `#{cmd} 2>/dev/null` `` executes arbitrary shell strings via Ruby backticks. The `track_dotfile` (line 25) and `restore_dotfile` (line 37) methods interpolate user-supplied `params[:path]` and `params[:name]` into shell commands after only minimal quote escaping:
  ```ruby
  safe_forge_command("forge grip dotfiles track \"#{path.gsub(/\"/, '\\\"')}\"")
  ```
  The `gsub` only escapes double quotes but not backticks, `$()`, `;`, `|`, `&`, or newlines.
- **Fix**: Use `Open3.capture3('forge', 'grip', 'dotfiles', 'track', path)` with argument arrays. Never interpolate user input into shell strings.

### C3. Command Injection in Rails `BridgeController#which?`
- **File**: `hub/app/controllers/bridge_controller.rb:119-121`
- **Category**: security
- **Description**: `system("which #{cmd} >/dev/null 2>&1")` interpolates `cmd` directly into a shell command. While currently called only with hardcoded strings, this is a public method that could be used with user input in the future.
- **Fix**: Use `system("which", cmd, ...)` array form or `Open3.capture3`.

### C4. Command Injection in Rails `FlameController#fetch_daily_verse`
- **File**: `hub/app/controllers/flame_controller.rb:93`
- **Category**: security
- **Description**: `` `forge word 2>/dev/null`.strip `` uses Ruby backticks to execute a shell command. While not currently injectable (no user input), it bypasses any argument safety.
- **Fix**: Use `Open3.capture3("forge", "word")`.

### C5. `unwrap()` on User-Controllable Shell Command Output in `bridge.rs`
- **File**: `src/bridge.rs:299`
- **Category**: bug
- **Description**: `stmt.query_map(...).unwrap()` inside the `run_sync` function will panic if the SQL query fails. This is in a user-facing code path (bridge sync), so a malformed DB or migration issue will crash the CLI.
- **Fix**: Use `?` or `map_err` to propagate the error.

### C6. `unwrap()` on User-Controllable String Parse in `mind.rs`
- **File**: `src/mind.rs:269`
- **Category**: bug
- **Description**: After `strip_prefix("name:")`, `.unwrap()` is called. If the string is somehow exactly `"name:"` (empty value), the strip_prefix returns `Some("")` but the unwrap is safe — however, the real issue is that this is parsing output from an external process (llama-swap config), and any unexpected format causes a panic.
- **Fix**: Use `if let Some(name) = trimmed.strip_prefix("name:") { ... }` or `.unwrap_or("")`.

---

## HIGH

### H1. Hardcoded Absolute Paths Throughout Codebase
- **Files**: `src/bridge.rs:37`, `hub/app/controllers/bridge_controller.rb:87`, `hub/app/controllers/bellows_controller.rb:152`
- **Category**: bug
- **Description**: Hardcoded path `/home/synth/llama.cpp/llama-swap/config.yaml` appears in multiple files. This makes the tool completely non-portable and will break on any other user's machine.
- **Fix**: Read from config or environment variable.

### H2. Memory Leak from `Box::leak` in Custom Theme Loading
- **File**: `src/theme.rs:317, 371`
- **Category**: bug
- **Description**: `Box::leak(name.into_boxed_str())` leaks heap memory every time custom themes are loaded. The `reload_custom_themes()` function at line 393-397 is a no-op — it admits the `OnceLock` can't be reset, so custom theme changes require a process restart. Each call to `to_theme()` leaks a new string.
- **Fix**: Use `Arc<str>` or store custom theme names in a static `Mutex<Vec<String>>`.

### H3. String Slicing on Non-ASCII Data Can Panic
- **File**: `src/bridge.rs:301, 333`
- **Category**: bug
- **Description**: `&row.1[..10]` slices a `String` by byte index. If the `created_at` field or agent name contains multi-byte UTF-8 characters (timestamps with unicode), this will panic with "byte index 10 is not a char boundary".
- **Fix**: Use `.chars().take(10).collect::<String>()` or truncate properly.

### H4. No Authentication on Rails Hub
- **File**: `hub/app/controllers/application_controller.rb`
- **Category**: security
- **Description**: No authentication middleware, no `before_action` for auth. Every endpoint is publicly accessible. The `trigger` and `restore` endpoints allow unauthenticated backup/restore operations. The `run_pipeline` and `strike` endpoints allow arbitrary command execution through the forge binary.
- **Fix**: Add authentication (Devise, HTTP basic auth, or IP allowlist for localhost-only deployment).

### H5. Race Condition in Backup Job Deduplication
- **File**: `hub/app/controllers/anvil/backups_controller.rb:25-27` and `hub/app/jobs/backup_job.rb:7-11`
- **Category**: bug
- **Description**: The "backup already running" check uses `Rails.cache.read("forge_backup_running")`. Between the check and the `perform_later` call, another request could also pass the check. The BackupJob itself also checks but has the same race. Using `perform_later` (async) means the job may not have started when the cache flag is written.
- **Fix**: Use `Rails.cache.write("forge_backup_running", true, unless_exist: true)` (atomic set-if-not-exists), or use a database-level advisory lock.

### H6. `Forge::Database` Opens New Connection Per Query, Never Closes on Success
- **File**: `hub/app/services/forge/database.rb:82-96`
- **Category**: bug
- **Description**: `with_retry` creates a new `SQLite3::Database` connection for each call but relies on Ruby's GC to close it. Under load, this leaks file descriptors. The `BellowsController` and `FlameController` manually `db.close` but `Forge::Database` does not — it just returns the block result.
- **Fix**: Use an `ensure` block to close the connection, or use a connection pool.

### H7. Unvalidated `params[:path]` in Backup Trigger
- **File**: `hub/app/controllers/anvil/backups_controller.rb:30`
- **Category**: security
- **Description**: `BackupJob.perform_later(path: params[:path])` passes user-supplied path directly to the forge binary's `quench` command. An attacker could pass `path: "/etc"` or any arbitrary directory to trigger a backup of sensitive data.
- **Fix**: Validate the path against a whitelist of configured repo paths.

### H8. `unwrap()` in `anvil.rs` Temper/Search on Temp Dir Path
- **File**: `src/anvil.rs:437`
- **Category**: bug
- **Description**: `temp_dir.path().to_str().unwrap()` can panic on non-UTF-8 temp paths. While rare on Linux, this is in a user-facing path.
- **Fix**: Use `.to_str().context("non-UTF-8 temp path")?`.

---

## MEDIUM

### M1. Excessive `rescue StandardError` Swallowing in Rails Controllers
- **Files**: `hub/app/controllers/flame_controller.rb:9-13,31-36,41-43,59-61`, `hub/app/controllers/bellows_controller.rb:7-10`, `hub/app/controllers/tongs_controller.rb:4-6`, `hub/app/controllers/crucible_controller.rb:10-13`
- **Category**: anti-pattern
- **Description**: Nearly every controller action has a blanket `rescue StandardError` that silently swallows all errors. This hides bugs, makes debugging impossible, and masks security issues. In production, 500 errors should be logged and reported, not silently returning empty data.
- **Fix**: Use `rescue => e` with `Rails.logger.error` at minimum. Only catch expected exceptions (e.g., `Forge::Database::NotFoundError`).

### M2. `Forge::Statistics` Loads All 10K Records Into Memory
- **File**: `hub/app/services/forge/statistics.rb:31,39,53,65`
- **Category**: anti-pattern
- **Description**: `top_repos`, `backup_frequency`, `disk_usage_trend`, and `weekly_trend` all call `@database.backups(limit: 10_000)` and then do in-memory aggregation. For large datasets, this loads thousands of rows into Ruby memory just to count or group them.
- **Fix**: Use SQL `GROUP BY` and aggregate functions directly in the database query.

### M3. `notify-send` Message Injection in Rust `bridge.rs`
- **File**: `src/bridge.rs:170-172`
- **Category**: security
- **Description**: User-supplied `message` is passed directly to `notify-send` via `.args(["Forge", message])`. While `Command.args` avoids shell injection, `notify-send` interprets certain characters specially (e.g., HTML in some desktop environments). This is minor since it's already using argument arrays.
- **Fix**: Sanitize or truncate the message. (Low priority.)

### M4. `BackupProgressChannel` Accepts Any `job_id` Param
- **File**: `hub/app/channels/backup_progress_channel.rb:3`
- **Category**: security
- **Description**: `stream_from "backup_progress_#{params[:job_id]}"` allows any user to subscribe to any backup job's progress stream by guessing or enumerating job IDs. No authorization check.
- **Fix**: Validate that the current user/session owns the job ID, or use signed/encrypted IDs.

### M5. Dead Code: `like` Variable Unused in SearchController
- **File**: `hub/app/controllers/search_controller.rb:32,79`
- **Category**: anti-pattern
- **Description**: `like = "%#{query}%"` is computed but never used in `search_backups` and `search_sessions`. The backup search does in-memory filtering instead, and the sessions search uses `like` but only from the second call.
- **Fix**: Remove the unused variable or use it for SQL LIKE queries.

### M6. Unnecessary `format!("{}")` Pattern
- **File**: `src/mind_cmd.rs:60,134` and elsewhere
- **Category**: anti-pattern
- **Description**: `format!("{}", routing.agent_type)` is redundant — just use `routing.agent_type.to_string()` or pass the value directly.
- **Fix**: Remove unnecessary `format!` wrappers.

### M7. `Reload` Custom Themes is a No-Op
- **File**: `src/theme.rs:393-397`
- **Category**: architecture
- **Description**: The function `reload_custom_themes()` exists but does nothing, with a comment admitting it. The OnceLock pattern means custom themes are loaded once per process and can never be refreshed.
- **Fix**: Remove the function and document the limitation, or switch to a `Mutex<HashMap>` pattern.

### M8. `find_forge_binary` Creates New `Forge::Client` Per Request
- **Files**: `hub/app/controllers/bellows_controller.rb:112-115`, `hub/app/controllers/crucible_controller.rb:87-91`, `hub/app/controllers/bridge_controller.rb:148-152`
- **Category**: anti-pattern
- **Description**: Every controller action that calls `find_forge_binary` creates a new `Forge::Client` instance, which runs binary resolution logic each time. This is wasteful.
- **Fix**: Cache the binary path in a class variable or initializer.

### M9. `FlameController#run_forge_command` Uses Shell String
- **File**: `hub/app/controllers/flame_controller.rb:156`
- **Category**: security
- **Description**: `Open3.capture3("forge", *args.map(&:to_s))` is mostly safe but there's no validation that `args` don't contain malicious content from user params (e.g., `query` in `search_scripture` is user-supplied and passed to the forge binary).
- **Fix**: Validate/sanitize query parameters before passing to CLI.

### M10. Test Unwraps in `spirit.rs`
- **File**: `src/spirit.rs:610,620,630,640,650`
- **Category**: anti-pattern
- **Description**: Multiple test functions call `result.unwrap()` on parsed Bible references. These are test-only, so panics are acceptable, but they indicate the `parse_reference` function can return `None` which isn't documented or handled in production code paths.
- **Fix**: Add `expect()` with messages or test the `None` case explicitly.

---

## LOW

### L1. `read_dir` in `bridge.rs` Silently Skips Errors
- **File**: `src/bridge.rs:109-111`
- **Category**: anti-pattern
- **Description**: `std::fs::read_dir(&hooks_dir)?.filter_map(|e| e.ok())` silently ignores directory entry read errors. Could hide permission issues.
- **Fix**: Log or report skipped entries.

### L2. `sudo` in `tongs.rs` Disk Health Check
- **File**: `src/tongs.rs:372`
- **Category**: anti-pattern
- **Description**: `sudo smartctl -H /dev/nvme0n1` requires root and will likely fail for most users. The fallback `|| sudo smartctl -H /dev/sda` compounds the issue.
- **Fix**: Document the requirement or use `smartctl --no-warranty` without sudo.

### L3. No Input Validation on `params[:cron_expression]`
- **File**: `hub/app/controllers/anvil/schedules_controller.rb:9`
- **Category**: test-gap
- **Description**: The cron expression is passed directly to the forge binary without validation. A malformed cron expression could cause unexpected behavior.
- **Fix**: Validate cron format before passing to the binary.

### L4. `params[:id]` in `delete_session` Not Validated as Integer
- **File**: `hub/app/controllers/bellows_controller.rb:56-58`
- **Category**: security
- **Description**: `id = params[:id]` is passed directly to `run_forge_command(["breathe", "sessions", "delete", id])`. While using argument arrays prevents shell injection, an attacker could pass arbitrary strings as session IDs.
- **Fix**: Validate `id` is a positive integer.

### L5. No Test Coverage for Critical Backup/Restore Rust Paths
- **Files**: `src/backup.rs`, `src/restore.rs`
- **Category**: test-gap
- **Description**: The core backup creation (`create_backup`, `create_incremental_backup`) and restore (`restore_backup`) functions have no unit tests. Only integration tests exist, and they're limited.
- **Fix**: Add unit tests with temporary directories and mock file systems.

### L6. No Test Coverage for `chunkstore.rs`
- **File**: `src/chunkstore.rs`
- **Category**: test-gap
- **Description**: The dedup chunk store has zero tests. This is a critical data integrity component (SHA-256 hashing, zstd compression, sharded paths).
- **Fix**: Add tests for chunk creation, retrieval, dedup, and corruption handling.

### L7. No Test Coverage for `scheduler.rs`
- **File**: `src/scheduler.rs`
- **Category**: test-gap
- **Description**: The cron scheduling module has no tests. Crontab generation and parsing are error-prone.
- **Fix**: Add tests for schedule CRUD and crontab generation.

### L8. Unused `Serialize`/`Deserialize` Derives May Exist
- **File**: `src/theme.rs:1` (imports `Serialize`, `Deserialize`)
- **Category**: anti-pattern
- **Description**: `serde::{Deserialize, Serialize}` is imported but only used for `CustomThemeDef`. The main `Theme` struct doesn't implement these traits, which means themes can't be serialized to JSON for API responses.
- **Fix**: Add `Serialize` to `Theme` if Hub integration is planned, or remove unused imports.

### L9. `color` Crate Import Unused
- **File**: `src/theme.rs:19-20` comment references `colored` crate bypass
- **Category**: anti-pattern
- **Description**: The theme system implements raw ANSI escape codes manually, bypassing the `colored` crate. If `colored` is still a dependency in `Cargo.toml`, it's dead weight.
- **Fix**: Remove `colored` from `Cargo.toml` if unused elsewhere.

### L10. No Rate Limiting on Expensive Hub Endpoints
- **Files**: `hub/app/controllers/anvil/backups_controller.rb:24-33`, `hub/app/controllers/flame_controller.rb:16-22`
- **Category**: security
- **Description**: The `trigger` backup endpoint and `search_scripture` endpoint have no rate limiting. An attacker could trigger thousands of backups or search queries.
- **Fix**: Add rate limiting middleware (e.g., `rack-attack`).

---

## Summary Statistics

| Severity | Count |
|----------|-------|
| CRITICAL | 6     |
| HIGH     | 8     |
| MEDIUM   | 10    |
| LOW      | 10    |
| **Total** | **34** |

### By Category

| Category      | Count |
|---------------|-------|
| Security      | 11    |
| Bug           | 7     |
| Anti-pattern  | 9     |
| Architecture  | 1     |
| Test-gap      | 4     |
| **Total**     | **34** |

### Top Priority Actions

1. **Fix command injection** in `TongsController` (C2) and `safe_command` paths (C1) — these are exploitable today.
2. **Add authentication** to the Rails Hub (H4) — all endpoints are publicly accessible.
3. **Remove hardcoded paths** (H1) — the tool breaks on any machine that isn't the developer's.
4. **Fix unwrap panics** on user-facing paths (C5, C6, H8).
5. **Fix string slicing** on potentially non-ASCII data (H3).
6. **Add tests** for backup/restore/chunkstore/scheduler (L5, L6, L7).
