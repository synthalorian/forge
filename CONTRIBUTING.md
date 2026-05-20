# Contributing to forge

Thanks for your interest in contributing to forge! Here's how to get started.

## Prerequisites

- Rust 1.75+ (stable)
- Git
- C compiler (for libgit2 and zstd native dependencies)

## Getting Started

```bash
git clone https://github.com/synthalorian/forge.git
cd forge
cargo build
cargo test
```

## Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Ensure all checks pass: `cargo fmt`, `cargo clippy`, `cargo test`
5. Commit with a descriptive message
6. Push and open a pull request

## Code Standards

- **Formatting:** Run `cargo fmt` before committing. CI enforces this.
- **Linting:** `cargo clippy -- -D warnings` must pass with zero warnings.
- **Error handling:** Use `anyhow::Result` for application code, `thiserror` for library-style error types in `error.rs`.
- **No `unwrap()`:** Use proper error propagation with `?` and `.context()`.
- **Documentation:** Public functions and modules should have doc comments explaining purpose and behavior.

## Project Structure

```
src/
├── main.rs       Entry point, CLI dispatch
├── cli.rs        Clap argument definitions
├── config.rs     Configuration loading and defaults
├── models.rs     Data models (BackupEntry, RepoSnapshot, etc.)
├── error.rs      Custom error types
├── db.rs         SQLite metadata database
├── backup.rs     Backup engine
├── archive.rs    Archive creation and extraction
├── restore.rs    Restore engine
└── scheduler.rs  Cron-based scheduling
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add incremental backup support
fix: handle missing reflog entries gracefully
docs: update README with restore examples
refactor: extract compression into archive module
test: add backup engine unit tests
```

## Reporting Issues

- Use GitHub Issues
- Include: Rust version, OS, steps to reproduce, expected vs actual behavior
- For crashes, include the full backtrace (set `RUST_BACKTRACE=1`)

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
