# frozen_string_literal: true

require "rails_helper"
require "tempfile"
require "fileutils"
require "sqlite3"

RSpec.describe Forge::Database do
  let(:test_db_path) { nil }
  let(:db_path) do
    temp = Tempfile.new(["forge_test_", ".db"])
    temp.close
    temp.path
  end
  let(:valid_db_path) { db_path }

  before do
    create_forge_db(db_path)
  end

  after do
    FileUtils.rm_f(db_path)
  end

  def create_forge_db(path)
    SQLite3::Database.open(path) do |conn|
      conn.execute(<<-SQL)
        CREATE TABLE backups (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          repo_path TEXT NOT NULL,
          repo_name TEXT NOT NULL,
          archive_path TEXT NOT NULL,
          sha256 TEXT NOT NULL,
          size_bytes INTEGER NOT NULL,
          branch_count INTEGER NOT NULL DEFAULT 0,
          tag_count INTEGER NOT NULL DEFAULT 0,
          commit_count INTEGER NOT NULL DEFAULT 0,
          backup_type TEXT NOT NULL DEFAULT 'full',
          created_at TEXT NOT NULL
        );
      SQL

      conn.execute(<<-SQL)
        CREATE TABLE schedules (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          cron_expression TEXT NOT NULL,
          target_path TEXT NOT NULL,
          enabled INTEGER NOT NULL DEFAULT 1,
          last_run TEXT,
          created_at TEXT NOT NULL
        );
      SQL

      conn.execute(
        "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ["/repo/a", "repo-a", "/archives/a.forge", "abc123", 1_000_000, 5, 3, 100, "full", "2024-01-01T00:00:00Z"]
      )
      conn.execute(
        "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ["/repo/b", "repo-b", "/archives/b.forge", "def456", 2_000_000, 2, 1, 50, "incremental", "2024-01-02T00:00:00Z"]
      )

      conn.execute(
        "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at) VALUES (?, ?, ?, ?, ?)",
        ["0 2 * * *", "/home/user/projects", 1, "2024-01-01T02:00:00Z", "2023-12-01T00:00:00Z"]
      )
      conn.execute(
        "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at) VALUES (?, ?, ?, ?, ?)",
        ["0 */6 * * *", "/home/user/docs", 0, nil, "2023-12-01T00:00:00Z"]
      )
    end
  end

  describe "#initialize" do
    it "accepts a custom path" do
      db = described_class.new(path: valid_db_path)
      expect(db.backup_count).to eq(2)
    end

    it "raises NotFoundError when database does not exist" do
      expect { described_class.new(path: "/nonexistent/path.db") }
        .to raise_error(Forge::Database::NotFoundError, /not found/)
    end

    it "raises CorruptedError when file is not a valid SQLite database" do
      corrupted = Tempfile.new(["corrupted_", ".db"])
      corrupted.write("not a sqlite database")
      corrupted.close

      db = described_class.new(path: corrupted.path)
      expect { db.backup_count }
        .to raise_error(Forge::Database::CorruptedError, /corrupted/)

      corrupted.unlink
    end
  end

  describe "#backups" do
    it "returns all backups ordered by created_at descending" do
      db = described_class.new(path: valid_db_path)
      results = db.backups

      expect(results.size).to eq(2)
      expect(results.first[:repo_name]).to eq("repo-b")
    end

    it "respects limit parameter" do
      db = described_class.new(path: valid_db_path)
      results = db.backups(limit: 1)
      expect(results.size).to eq(1)
    end

    it "respects offset parameter" do
      db = described_class.new(path: valid_db_path)
      results = db.backups(limit: 1, offset: 1)
      expect(results.size).to eq(1)
      expect(results.first[:repo_name]).to eq("repo-a")
    end

    it "returns backup with all expected keys" do
      db = described_class.new(path: valid_db_path)
      result = db.backups.first

      expect(result).to have_key(:id)
      expect(result).to have_key(:repo_path)
      expect(result).to have_key(:repo_name)
      expect(result).to have_key(:archive_path)
      expect(result).to have_key(:sha256)
      expect(result).to have_key(:size_bytes)
      expect(result).to have_key(:branch_count)
      expect(result).to have_key(:tag_count)
      expect(result).to have_key(:commit_count)
      expect(result).to have_key(:backup_type)
      expect(result).to have_key(:created_at)
    end
  end

  describe "#backup_count" do
    it "returns total count of backups" do
      db = described_class.new(path: valid_db_path)
      expect(db.backup_count).to eq(2)
    end

    it "returns 0 when no backups exist" do
      empty_db = Tempfile.new(["empty_", ".db"])
      empty_db.close
      SQLite3::Database.open(empty_db.path) do |conn|
        conn.execute("CREATE TABLE IF NOT EXISTS backups (id INTEGER PRIMARY KEY, repo_path TEXT NOT NULL, repo_name TEXT NOT NULL, archive_path TEXT NOT NULL, sha256 TEXT NOT NULL, size_bytes INTEGER NOT NULL, branch_count INTEGER NOT NULL DEFAULT 0, tag_count INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, backup_type TEXT NOT NULL DEFAULT 'full', created_at TEXT NOT NULL)")
        conn.execute("CREATE TABLE IF NOT EXISTS schedules (id INTEGER PRIMARY KEY, cron_expression TEXT NOT NULL, target_path TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1, last_run TEXT, created_at TEXT NOT NULL)")
      end
      db = described_class.new(path: empty_db.path)
      expect(db.backup_count).to eq(0)
      empty_db.unlink
    end
  end

  describe "#find_backup" do
    it "finds a backup by id" do
      db = described_class.new(path: valid_db_path)
      result = db.find_backup(1)

      expect(result).to_not be_nil
      expect(result[:repo_name]).to eq("repo-a")
    end

    it "returns nil when backup not found" do
      db = described_class.new(path: valid_db_path)
      expect(db.find_backup(999)).to be_nil
    end
  end

  describe "#schedules" do
    it "returns all schedules" do
      db = described_class.new(path: valid_db_path)
      results = db.schedules

      expect(results.size).to eq(2)
    end

    it "returns schedule with expected keys" do
      db = described_class.new(path: valid_db_path)
      result = db.schedules.first

      expect(result).to have_key(:id)
      expect(result).to have_key(:cron_expression)
      expect(result).to have_key(:target_path)
      expect(result).to have_key(:enabled)
      expect(result).to have_key(:last_run)
      expect(result).to have_key(:created_at)
    end

    it "returns enabled as boolean" do
      db = described_class.new(path: valid_db_path)
      results = db.schedules

      expect(results[0][:enabled]).to be true
      expect(results[1][:enabled]).to be false
    end
  end

  describe "#disk_usage" do
    it "returns sum of all backup sizes" do
      db = described_class.new(path: valid_db_path)
      expect(db.disk_usage).to eq(3_000_000)
    end
  end

  describe "#unique_repos" do
    it "returns count of distinct repos" do
      db = described_class.new(path: valid_db_path)
      expect(db.unique_repos).to eq(2)
    end
  end
end
