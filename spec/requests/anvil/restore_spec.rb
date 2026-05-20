require "rails_helper"

RSpec.describe "Anvil::Restore", type: :request do
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(backups: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["anvil_restore_test_", ".db"])
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

  before(:each) do
    Rails.cache.clear
  end

  describe "POST /anvil/backups/:id/restore" do
    it "starts restore job and redirects" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(RestoreJob).to receive(:perform_later)

      post "/anvil/backups/1/restore"
      expect(response).to redirect_to(anvil_backup_path(1))
      expect(flash[:notice]).to include("Restore started")
    end

    it "shows error for nonexistent backup" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      post "/anvil/backups/999999/restore"
      expect(response).to redirect_to(anvil_backups_path)
      expect(flash[:alert]).to eq("Backup not found.")
    end

    it "prevents concurrent restore of same backup" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_restore_running_1").and_return(true)

      post "/anvil/backups/1/restore"
      expect(response).to redirect_to(anvil_backup_path(1))
      expect(flash[:alert]).to eq("Restore already in progress for this backup.")
    end
  end

  describe "GET /anvil/backups/:id (restore button and status)" do
    it "shows restore button on detail page" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      get "/anvil/backups/1"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Restore Backup")
    end

    it "shows restore in progress status" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_restore_running_1").and_return(true)

      get "/anvil/backups/1"
      expect(response.body).to include("Restore in progress")
    end

    it "shows restore success status" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_restore_result_1").and_return({
        "status" => "success",
        "output" => "done"
      })

      get "/anvil/backups/1"
      expect(response.body).to include("Restore complete")
    end

    it "shows restore error status" do
      path = create_db(backups: [{
        repo_path: "/repo/test", repo_name: "test-repo",
        archive_path: "/arc/test.git.forge",
        sha256: "aabbccdd", size_bytes: 1_048_576,
        created_at: "2026-05-20T12:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_restore_result_1").and_return({
        "status" => "error",
        "message" => "Restore failed: disk full"
      })

      get "/anvil/backups/1"
      expect(response.body).to include("Restore failed: disk full")
    end
  end
end
