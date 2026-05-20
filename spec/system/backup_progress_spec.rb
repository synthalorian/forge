# frozen_string_literal: true

require "rails_helper"

RSpec.describe "Backup Progress", type: :system do
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_test_db
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["backup_progress_test_", ".db"])
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
    end
    temp.path
  end

  before(:each) do
    @db_path = create_test_db
    ENV["FORGE_DB_PATH"] = @db_path
  end

  describe "backup trigger and progress display" do
    it "shows Backup All button on backups page" do
      visit "/anvil/backups"
      expect(page).to have_button("Backup All")
    end

    it "shows progress container when backup is running with job id in session" do
      visit "/anvil/backups"
      expect(page).to have_content("BACKUPS")
    end

    it "displays completion status after backup finishes" do
      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_backup_running").and_return(nil)
      allow(Rails.cache).to receive(:read).with("forge_backup_result").and_return(
        { "status" => "success", "message" => "Backup complete!" }
      )

      visit "/anvil/backups"

      expect(page).to have_content("Backup completed successfully")
    end

    it "displays error status when backup fails" do
      allow(Rails.cache).to receive(:read).and_call_original
      allow(Rails.cache).to receive(:read).with("forge_backup_running").and_return(nil)
      allow(Rails.cache).to receive(:read).with("forge_backup_result").and_return(
        { "status" => "error", "message" => "Something went wrong" }
      )

      visit "/anvil/backups"

      expect(page).to have_content("Backup failed")
      expect(page).to have_content("Something went wrong")
    end
  end

  describe "BackupJob streaming" do
    it "broadcasts output lines via Turbo Streams" do
      allow(Forge::Client).to receive(:new).and_return(
        instance_double(Forge::Client, bin_path: "echo")
      )

      expect(Turbo::StreamsChannel).to receive(:broadcast_replace_to).at_least(:once)
      expect(Turbo::StreamsChannel).to receive(:broadcast_append_to).at_least(:once)

      BackupJob.new.perform(job_id: "test-streaming-job")
    end

    it "broadcasts error status on failure" do
      allow(Forge::Client).to receive(:new).and_raise("Binary not found")

      expect(Turbo::StreamsChannel).to receive(:broadcast_replace_to).at_least(:once)

      BackupJob.new.perform(job_id: "test-error-job")
    end

    it "deletes the running flag after completion" do
      client = instance_double(Forge::Client, bin_path: "echo")
      allow(Forge::Client).to receive(:new).and_return(client)
      allow(Turbo::StreamsChannel).to receive(:broadcast_append_to)
      allow(Turbo::StreamsChannel).to receive(:broadcast_replace_to)

      BackupJob.new.perform(job_id: "test-cleanup-job")

      expect(Rails.cache.read("forge_backup_running")).to be_nil
    end

    it "writes result to cache on success" do
      client = instance_double(Forge::Client, bin_path: "echo")
      allow(Forge::Client).to receive(:new).and_return(client)
      allow(Turbo::StreamsChannel).to receive(:broadcast_append_to)
      allow(Turbo::StreamsChannel).to receive(:broadcast_replace_to)
      allow(Rails.cache).to receive(:write).and_call_original

      BackupJob.new.perform(job_id: "test-success-job")

      expect(Rails.cache).to have_received(:write).with("forge_backup_result", hash_including("status" => "success"))
    end
  end
end
