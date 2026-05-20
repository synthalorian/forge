require "rails_helper"

RSpec.describe "Anvil::Backups Browse", type: :request do
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(backups: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["anvil_browse_test_", ".db"])
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

  let(:sample_backup) do
    {
      repo_path: "/repo/test", repo_name: "test-repo",
      archive_path: "/tmp/nonexistent_archive.tar.zst",
      sha256: "aabbccdd", size_bytes: 1_048_576,
      branch_count: 3, tag_count: 2, commit_count: 50,
      backup_type: "full",
      created_at: "2026-05-20T12:00:00Z"
    }
  end

  describe "GET /anvil/backups/:id/browse" do
    it "returns 200 with archive file listing" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      # Stub the archive listing since we can't rely on zstd in CI
      allow_any_instance_of(Anvil::BackupsController).to receive(:list_archive_contents).and_return({
        entries: [
          { path: "test-repo.git", directory: true },
          { path: "test-repo.git/HEAD", directory: false },
          { path: "test-repo.git/objects", directory: true },
          { path: "test-repo.git/refs", directory: true }
        ],
        error: nil,
        total_count: 4
      })

      get "/anvil/backups/1/browse"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("BROWSE")
      expect(response.body).to include("test-repo.git")
      expect(response.body).to include("4 entries")
      expect(response.body).to include("Back to Backup Detail")
    end

    it "returns 404 for nonexistent backup" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path
      get "/anvil/backups/999999/browse"
      expect(response).to have_http_status(:not_found)
    end

    it "shows archive missing message when file not found" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      # The archive path doesn't exist, so list_archive_contents returns archive_missing
      get "/anvil/backups/1/browse"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Archive Not Found")
      expect(response.body).to include("/tmp/nonexistent_archive.tar.zst")
    end

    it "shows metadata and warning when zstd not installed" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      allow_any_instance_of(Anvil::BackupsController).to receive(:list_archive_contents).and_return({
        entries: [],
        error: :zstd_not_installed
      })

      get "/anvil/backups/1/browse"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("zstd not installed")
    end

    it "displays breadcrumb with repo name and browse" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      allow_any_instance_of(Anvil::BackupsController).to receive(:list_archive_contents).and_return({
        entries: [{ path: "test-repo.git", directory: true }],
        error: nil,
        total_count: 1
      })

      get "/anvil/backups/1/browse"
      expect(response.body).to include("Browse")
      expect(response.body).to include("Backups")
    end

    it "displays backup metadata in header" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      allow_any_instance_of(Anvil::BackupsController).to receive(:list_archive_contents).and_return({
        entries: [{ path: "test-repo.git", directory: true }],
        error: nil,
        total_count: 1
      })

      get "/anvil/backups/1/browse"
      expect(response.body).to include("test-repo")
      expect(response.body).to include("50 commits")
      expect(response.body).to include("3 branches")
      expect(response.body).to include("2 tags")
    end

    it "handles tar/zstd failure gracefully" do
      path = create_db(backups: [sample_backup])
      ENV["FORGE_DB_PATH"] = path

      allow_any_instance_of(Anvil::BackupsController).to receive(:list_archive_contents).and_return({
        entries: [],
        error: :tar_failed,
        message: "tar: This does not look like a tar archive"
      })

      get "/anvil/backups/1/browse"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Unable to Read Archive")
      expect(response.body).to include("tar: This does not look like a tar archive")
    end
  end
end
