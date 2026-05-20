require "rails_helper"

RSpec.describe "Anvil::BackupTrigger", type: :request do
  let(:db_path) { nil }
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(backups: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["anvil_trigger_test_", ".db"])
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

  describe "POST /anvil/backups/trigger" do
    it "triggers a backup job" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      allow(BackupJob).to receive(:perform_later)

      post "/anvil/backups/trigger"
      expect(response).to redirect_to(anvil_backups_path)
      expect(flash[:notice]).to eq("Backup started.")
    end

    it "prevents concurrent backups" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_backup_running").and_return(true)

      post "/anvil/backups/trigger"
      expect(response).to redirect_to(anvil_backups_path)
      expect(flash[:alert]).to eq("A backup is already in progress.")
    end

    it "shows backup button on list page" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      get "/anvil/backups"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Backup All")
    end
  end
end
