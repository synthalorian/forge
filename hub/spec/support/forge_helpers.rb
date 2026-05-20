# frozen_string_literal: true

require "fileutils"
require "tempfile"

module ForgeHelpers
  FORGE_BINARY_PATH = File.expand_path("~/.local/bin/forge").freeze
  FORGE_DB_PATH = File.expand_path("~/.local/share/forge/forge.db").freeze

  def with_mock_forge(responses = {})
    defaults = {
      "version" => "1.0.0",
      "list --json" => [].to_json,
      "status" => "ok"
    }.merge(responses)

    allow(File).to receive(:exist?).with(FORGE_BINARY_PATH).and_return(true)
    allow(Open3).to receive(:capture3).and_wrap_original do |method, *args|
      cmd = args.join(" ")
      response_key = defaults.keys.find { |k| cmd.include?(k) } || "status"
      stdout, stderr, status = defaults[response_key], "", double(success?: true)
      [ stdout, stderr, status ]
    end

    yield
  end

  def without_forge
    allow(File).to receive(:exist?).with(FORGE_BINARY_PATH).and_return(false)
    yield
  end

  def with_test_db(forge_db_path = nil)
    db_path = forge_db_path || ForgeHelpers.create_temp_forge_db
    ENV["FORGE_DB_PATH"] = db_path
    yield db_path
  ensure
    ENV.delete("FORGE_DB_PATH")
    FileUtils.rm_f(db_path) if forge_db_path.nil? && db_path
  end

  def self.create_temp_forge_db(backups: [], schedules: [])
    db = Tempfile.new([ "forge_test_", ".db" ])
    db.close

    require "sqlite3"
    SQLite3::Database.open(db.path) do |conn|
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

      now = Time.now.utc.iso8601

      backups.each do |backup|
        conn.execute(
          "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes,
           branch_count, tag_count, commit_count, backup_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
          backup[:repo_path], backup[:repo_name], backup[:archive_path], backup[:sha256],
          backup[:size_bytes], backup[:branch_count] || 0, backup[:tag_count] || 0,
          backup[:commit_count] || 0, backup[:backup_type] || "full", backup[:created_at] || now
        )
      end

      schedules.each do |schedule|
        conn.execute(
          "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at) VALUES (?, ?, ?, ?, ?)",
          schedule[:cron_expression], schedule[:target_path], schedule[:enabled] ? 1 : 0,
          schedule[:last_run], schedule[:created_at] || now
        )
      end
    end

    db.path
  end
end

RSpec.configure do |config|
  config.include ForgeHelpers, type: :system
  config.include ForgeHelpers, type: :request
  config.include ForgeHelpers, type: :controller
end
