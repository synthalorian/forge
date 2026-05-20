require "rails_helper"

RSpec.describe "Anvil::Backups", type: :request do
  let(:db_path) { nil }
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(backups: [], schedules: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["anvil_test_", ".db"])
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

  describe "GET /anvil/backups" do
    it "returns 200" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups"
      expect(response).to have_http_status(:ok)
    end

    it "renders backup list with data" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        branch_count: 3, tag_count: 2, commit_count: 50,
        backup_type: "full",
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups"
      expect(response.body).to include("test-repo")
      expect(response.body).to include("BACKUPS")
    end

    it "renders empty state when no backups" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups"
      expect(response.body).to include("No backups")
    end

    it "renders pagination when more than 25 backups" do
      backups = (1..30).map do |i|
        { repo_path: "/repo/repo#{i}", repo_name: "repo#{i}",
          archive_path: "/arc/repo#{i}.forge", sha256: "abc#{i}",
          size_bytes: 100_000, created_at: "2026-05-20T12:00:00Z" }
      end
      path = create_db(backups: backups)
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups"
      expect(response.body).to include("Page 1")
      expect(response.body).to include("Next")
    end

    it "shows next page" do
      backups = (1..30).map do |i|
        { repo_path: "/repo/repo#{i}", repo_name: "repo#{i}",
          archive_path: "/arc/repo#{i}.forge", sha256: "abc#{i}",
          size_bytes: 100_000, created_at: "2026-05-20T12:00:00Z" }
      end
      path = create_db(backups: backups)
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups", params: { page: 2 }
      expect(response.body).to include("Previous")
    end
  end

  describe "GET /anvil/backups/:id" do
    it "returns 200 for existing backup" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        branch_count: 3, tag_count: 2, commit_count: 50,
        backup_type: "full",
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/1"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("test-repo")
      expect(response.body).to include("aabbccdd")
    end

    it "returns 404 for nonexistent backup" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/999999"
      expect(response).to have_http_status(:not_found)
    end

    it "renders all metadata fields" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        branch_count: 3, tag_count: 2, commit_count: 50,
        backup_type: "full",
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/1"
      expect(response.body).to include("SHA-256")
      expect(response.body).to include("Created")
      expect(response.body).to include("Size")
      expect(response.body).to include("Commits")
      expect(response.body).to include("Branches")
      expect(response.body).to include("Tags")
    end
  end

  describe "GET /anvil/schedules" do
    it "returns 200" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/schedules"
      expect(response).to have_http_status(:ok)
    end

    it "renders schedule list with data" do
      path = create_db(schedules: [{
        cron_expression: "0 2 * * *",
        target_path: "/home/user/projects",
        enabled: true,
        last_run: "2026-05-20T02:00:00Z",
        created_at: "2026-05-01T00:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/schedules"
      expect(response.body).to include("SCHEDULES")
      expect(response.body).to include("0 2 * * *")
      expect(response.body).to include("/home/user/projects")
    end

    it "renders empty state when no schedules" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/schedules"
      expect(response.body).to include("No schedules")
    end
  end

  describe "GET /anvil" do
    it "redirects to /anvil/backups" do
      get "/anvil"
      expect(response).to redirect_to("/anvil/backups")
    end
  end
end