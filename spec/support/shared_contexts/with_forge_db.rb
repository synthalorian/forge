# frozen_string_literal: true

require "fileutils"
require "tempfile"
require "sqlite3"

RSpec.shared_context "with_forge_db", with_forge_db: true do
  include ForgeHelpers
  include DatabaseFixtures

  let(:temp_forge_db_path) do
    @temp_db = Tempfile.new([ "forge_test_", ".db" ])
    @temp_db.close

    SQLite3::Database.open(@temp_db.path) do |conn|
      conn.execute(<<-SQL)
        CREATE TABLE backups (
          id              INTEGER PRIMARY KEY AUTOINCREMENT,
          repo_path       TEXT NOT NULL,
          repo_name       TEXT NOT NULL,
          archive_path    TEXT NOT NULL,
          sha256          TEXT NOT NULL,
          size_bytes      INTEGER NOT NULL,
          branch_count    INTEGER NOT NULL DEFAULT 0,
          tag_count       INTEGER NOT NULL DEFAULT 0,
          commit_count    INTEGER NOT NULL DEFAULT 0,
          backup_type     TEXT NOT NULL DEFAULT 'full',
          created_at      TEXT NOT NULL
        );
      SQL

      conn.execute(<<-SQL)
        CREATE TABLE schedules (
          id              INTEGER PRIMARY KEY AUTOINCREMENT,
          cron_expression TEXT NOT NULL,
          target_path     TEXT NOT NULL,
          enabled         INTEGER NOT NULL DEFAULT 1,
          last_run        TEXT,
          created_at      TEXT NOT NULL
        );
      SQL

      conn.execute("CREATE INDEX idx_backups_repo_name ON backups(repo_name);")
      conn.execute("CREATE INDEX idx_backups_created_at ON backups(created_at);")

      conn.execute(
        "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        "/home/user/projects/sample-repo",
        "sample-repo",
        "/tmp/forge/archives/sample-repo-001.forge",
        "abc123def456789",
        10_485_760,
        3,
        2,
        50,
        "full",
        Time.now.utc.iso8601
      )

      conn.execute(
        "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at)
         VALUES (?, ?, ?, ?, ?)",
        "0 2 * * *",
        "/home/user/projects",
        1,
        Time.now.utc.iso8601,
        Time.now.utc.iso8601
      )
    end

    @temp_db.path
  end

  around(:each) do |example|
    original_db_path = ENV["FORGE_DB_PATH"]
    ENV["FORGE_DB_PATH"] = temp_forge_db_path
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = original_db_path
    @temp_db&.close
    FileUtils.rm_f(temp_forge_db_path)
  end
end
