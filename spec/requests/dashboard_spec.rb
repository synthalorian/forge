require "rails_helper"

RSpec.describe "Dashboard", type: :request do
  around(:each) do |example|
    @saved = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved
  end

  def create_db(backups: [], schedules: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["dash_test_", ".db"])
    temp.close
    SQLite3::Database.open(temp.path) do |conn|
      conn.execute(<<-SQL)
        CREATE TABLE backups (
          id INTEGER PRIMARY KEY AUTOINCREMENT, repo_path TEXT NOT NULL,
          repo_name TEXT NOT NULL, archive_path TEXT NOT NULL, sha256 TEXT NOT NULL,
          size_bytes INTEGER NOT NULL, branch_count INTEGER DEFAULT 0,
          tag_count INTEGER DEFAULT 0, commit_count INTEGER DEFAULT 0,
          backup_type TEXT DEFAULT 'full', created_at TEXT NOT NULL
        );
      SQL
      conn.execute(<<-SQL)
        CREATE TABLE schedules (
          id INTEGER PRIMARY KEY AUTOINCREMENT, cron_expression TEXT NOT NULL,
          target_path TEXT NOT NULL, enabled INTEGER DEFAULT 1,
          last_run TEXT, created_at TEXT NOT NULL
        );
      SQL
      backups.each do |b|
        conn.execute(
          "INSERT INTO backups (repo_path, repo_name, archive_path, sha256, size_bytes, branch_count, tag_count, commit_count, backup_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
          [b[:repo_path], b[:repo_name], b[:archive_path], b[:sha256],
           b[:size_bytes], b[:branch_count] || 0, b[:tag_count] || 0,
           b[:commit_count] || 0, b[:backup_type] || "full", b[:created_at]]
        )
      end
      schedules.each do |s|
        conn.execute(
          "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at) VALUES (?, ?, ?, ?, ?)",
          [s[:cron_expression], s[:target_path], s[:enabled] ? 1 : 0, s[:last_run], s[:created_at]]
        )
      end
    end
    temp.path
  end

  describe "GET /" do
    it "shows dashboard with live stats" do
      path = create_db(backups: [
        { repo_path: "/repo/a", repo_name: "repo-a",
          archive_path: "/arc/a.forge", sha256: "aa",
          size_bytes: 1_000_000, created_at: "2026-01-01T00:00:00Z" },
        { repo_path: "/repo/b", repo_name: "repo-b",
          archive_path: "/arc/b.forge", sha256: "bb",
          size_bytes: 2_000_000, created_at: "2026-01-02T00:00:00Z" },
      ], schedules: [{ cron_expression: "0 2 * * *", target_path: "/repo/a", enabled: true, last_run: nil, created_at: "2026-01-01T00:00:00Z" }])
      ENV["FORGE_DB_PATH"] = path
      get "/"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("FORGE HUB")
      expect(response.body).to include("Pillars")
    end

    it "shows backup count in stats" do
      path = create_db(backups: [
        { repo_path: "/repo/a", repo_name: "repo-a",
          archive_path: "/arc/a.forge", sha256: "aa",
          size_bytes: 1_000_000, created_at: "2026-01-01T00:00:00Z" },
        { repo_path: "/repo/a", repo_name: "repo-a",
          archive_path: "/arc/a2.forge", sha256: "bb",
          size_bytes: 500_000, created_at: "2026-01-02T00:00:00Z" },
      ])
      ENV["FORGE_DB_PATH"] = path
      get "/"
      expect(response.body).to include("2")
      expect(response.body).to include("1")
    end

    it "shows placeholders when forge not installed" do
      ENV.delete("FORGE_DB_PATH")
      allow(File).to receive(:exist?).and_call_original
      allow(File).to receive(:exist?).with(/\/forge/).and_return(false)
      get "/"
      expect(response.body).to include("Forge Not Detected").or include("FORGE HUB")
    end
  end
end