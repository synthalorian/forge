require "rails_helper"

RSpec.describe "Anvil::Backups Chart Data", type: :request do
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(backups: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["chart_test_", ".db"])
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
    end
    temp.path
  end

  describe "GET /anvil/backups/chart_data" do
    it "returns JSON with empty array when no backups" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/chart_data", params: { format: :json }
      expect(response).to have_http_status(:ok)
      json = JSON.parse(response.body)
      expect(json).to eq([])
    end

    it "returns grouped backup data by date" do
      path = create_db(backups: [
        { repo_path: "/repo/forge", repo_name: "forge",
          archive_path: "/arc/forge.forge", sha256: "aaa111",
          size_bytes: 1_048_576, created_at: "2026-05-19T10:00:00Z" },
        { repo_path: "/repo/forge", repo_name: "forge",
          archive_path: "/arc/forge2.forge", sha256: "aaa222",
          size_bytes: 2_097_152, created_at: "2026-05-19T14:00:00Z" },
        { repo_path: "/repo/hub", repo_name: "hub",
          archive_path: "/arc/hub.forge", sha256: "bbb111",
          size_bytes: 512_000, created_at: "2026-05-20T08:00:00Z" }
      ])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/chart_data", params: { format: :json }
      expect(response).to have_http_status(:ok)
      json = JSON.parse(response.body)
      expect(json.length).to eq(2)

      may19 = json.find { |d| d["date"] == "2026-05-19" }
      expect(may19).to be_present
      expect(may19["size"]).to eq(3_145_728) # 1MB + 2MB
      expect(may19["repo_name"]).to eq("forge")

      may20 = json.find { |d| d["date"] == "2026-05-20" }
      expect(may20).to be_present
      expect(may20["size"]).to eq(512_000)
    end

    it "returns empty array when forge database not found" do
      ENV["FORGE_DB_PATH"] = "/tmp/nonexistent_forge_test.db"
      get "/anvil/backups/chart_data", params: { format: :json }
      expect(response).to have_http_status(:ok)
      json = JSON.parse(response.body)
      expect(json).to eq([])
    end

    it "excludes backups older than 30 days" do
      path = create_db(backups: [
        { repo_path: "/repo/old", repo_name: "old",
          archive_path: "/arc/old.forge", sha256: "old111",
          size_bytes: 999_999, created_at: "2026-04-01T10:00:00Z" },
        { repo_path: "/repo/new", repo_name: "new",
          archive_path: "/arc/new.forge", sha256: "new111",
          size_bytes: 500_000, created_at: "2026-05-20T10:00:00Z" }
      ])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/chart_data", params: { format: :json }
      json = JSON.parse(response.body)
      dates = json.map { |d| d["date"] }
      expect(dates).not_to include("2026-04-01")
      expect(dates).to include("2026-05-20")
    end
  end
end
