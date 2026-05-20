# frozen_string_literal: true

require "fileutils"
require "tempfile"
require "sqlite3"

module DatabaseFixtures
  class ForgeDBBuilder
    attr_reader :backups, :schedules

    def initialize
      @backups = []
      @schedules = []
    end

    def add_backup(attributes)
      @backups << {
        repo_path: attributes[:repo_path] || "/home/user/projects/test-repo",
        repo_name: attributes[:repo_name] || "test-repo",
        archive_path: attributes[:archive_path] || "/tmp/forge/archives/test-repo-001.forge",
        sha256: attributes[:sha256] || SecureRandom.hex(32),
        size_bytes: attributes[:size_bytes] || 1_024_576,
        branch_count: attributes[:branch_count] || 3,
        tag_count: attributes[:tag_count] || 2,
        commit_count: attributes[:commit_count] || 47,
        backup_type: attributes[:backup_type] || "full",
        created_at: attributes[:created_at] || Time.now.utc.iso8601
      }
      self
    end

    def add_schedule(attributes)
      @schedules << {
        cron_expression: attributes[:cron_expression] || "0 2 * * *",
        target_path: attributes[:target_path] || "/home/user/projects",
        enabled: attributes.fetch(:enabled, true),
        last_run: attributes[:last_run],
        created_at: attributes[:created_at] || Time.now.utc.iso8601
      }
      self
    end

    def build(path = nil)
      db_path = path || create_temp_db_path
      create_database(db_path)
      db_path
    end

    private

    def create_temp_db_path
      temp_db = Tempfile.new([ "forge_fixture_", ".db" ])
      temp_db.close
      temp_db.path
    end

    def create_database(path)
      SQLite3::Database.open(path) do |conn|
        create_tables(conn)
        insert_data(conn)
      end
      path
    end

    def create_tables(conn)
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
    end

    def insert_data(conn)
      @backups.each do |backup|
        conn.execute(
          "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes,
           branch_count, tag_count, commit_count, backup_type, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
          backup[:repo_path], backup[:repo_name], backup[:archive_path],
          backup[:sha256], backup[:size_bytes], backup[:branch_count],
          backup[:tag_count], backup[:commit_count], backup[:backup_type], backup[:created_at]
        )
      end

      @schedules.each do |schedule|
        conn.execute(
          "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at)
           VALUES (?, ?, ?, ?, ?)",
          schedule[:cron_expression], schedule[:target_path],
          schedule[:enabled] ? 1 : 0, schedule[:last_run], schedule[:created_at]
        )
      end
    end
  end

  def self.sample_forge_db
    builder = ForgeDBBuilder.new
    builder.add_backup(
      repo_path: "/home/user/projects/my-awesome-app",
      repo_name: "my-awesome-app",
      archive_path: "/tmp/forge/archives/my-awesome-app-20240520-120000.forge",
      sha256: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
      size_bytes: 52_428_800,
      branch_count: 5,
      tag_count: 12,
      commit_count: 234,
      backup_type: "full"
    )
    builder.add_backup(
      repo_path: "/home/user/projects/api-service",
      repo_name: "api-service",
      archive_path: "/tmp/forge/archives/api-service-20240519-180000.forge",
      sha256: "f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5",
      size_bytes: 28_311_040,
      branch_count: 3,
      tag_count: 4,
      commit_count: 89,
      backup_type: "full"
    )
    builder.add_schedule(
      cron_expression: "0 2 * * *",
      target_path: "/home/user/projects",
      enabled: true,
      last_run: "2024-05-20T02:00:00Z"
    )
    builder.add_schedule(
      cron_expression: "0 */6 * * *",
      target_path: "/home/user/documents",
      enabled: false,
      last_run: nil
    )
    builder.build
  end

  def with_database_fixtures
    @temp_db_path = DatabaseFixtures.sample_forge_db
    ENV["FORGE_DB_PATH"] = @temp_db_path
    yield @temp_db_path
  ensure
    ENV.delete("FORGE_DB_PATH")
    FileUtils.rm_f(@temp_db_path) if @temp_db_path
  end
end

RSpec.configure do |config|
  config.include DatabaseFixtures
end
