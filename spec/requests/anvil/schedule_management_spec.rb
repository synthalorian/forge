require "rails_helper"

RSpec.describe "Anvil::ScheduleManagement", type: :request do
  let(:db_path) { nil }
  around(:each) do |example|
    @saved_forge_db_path = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_forge_db_path
  end

  def create_db(schedules: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["anvil_schedule_test_", ".db"])
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
      schedules.each do |s|
        conn.execute(
          "INSERT INTO schedules (cron_expression, target_path, enabled, last_run, created_at) VALUES (?, ?, ?, ?, ?)",
          [s[:cron_expression], s[:target_path], s[:enabled] ? 1 : 0, s[:last_run], s[:created_at]]
        )
      end
    end
    temp.path
  end

  let(:mock_client) { double("Forge::Client") }

  before(:each) do
    allow(Forge::Client).to receive(:new).and_return(mock_client)
  end

  describe "POST /anvil/schedules" do
    it "adds a schedule" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      allow(mock_client).to receive(:schedule_add)

      post "/anvil/schedules", params: { cron_expression: "0 2 * * *", target_path: "/repo/test" }
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:notice]).to eq("Schedule added.")
    end

    it "rejects missing cron expression" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      post "/anvil/schedules", params: { target_path: "/repo/test" }
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:alert]).to include("required")
    end

    it "rejects missing target path" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      post "/anvil/schedules", params: { cron_expression: "0 2 * * *" }
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:alert]).to include("required")
    end

    it "handles forge CLI errors" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      allow(mock_client).to receive(:schedule_add).and_raise(Forge::Client::CommandError, "invalid cron")

      post "/anvil/schedules", params: { cron_expression: "bad", target_path: "/repo/test" }
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:alert]).to include("Failed to add schedule")
    end
  end

  describe "DELETE /anvil/schedules/:id" do
    it "removes a schedule" do
      path = create_db(schedules: [{
        cron_expression: "0 2 * * *",
        target_path: "/repo/test",
        enabled: true,
        last_run: nil,
        created_at: "2026-05-20T00:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      allow(mock_client).to receive(:schedule_remove)

      delete "/anvil/schedules/1"
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:notice]).to eq("Schedule removed.")
    end
  end

  describe "PATCH /anvil/schedules/:id/toggle" do
    it "shows notice about CLI version requirement" do
      path = create_db(schedules: [{
        cron_expression: "0 2 * * *",
        target_path: "/repo/test",
        enabled: true,
        last_run: nil,
        created_at: "2026-05-20T00:00:00Z"
      }])
      ENV["FORGE_DB_PATH"] = path

      patch "/anvil/schedules/1/toggle"
      expect(response).to redirect_to(anvil_schedules_path)
      expect(flash[:notice]).to include("forge CLI v0.2+")
    end
  end

  describe "GET /anvil/schedules" do
    it "shows add form on index" do
      path = create_db
      ENV["FORGE_DB_PATH"] = path

      get "/anvil/schedules"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Add Schedule")
      expect(response.body).to include("cron_expression")
      expect(response.body).to include("target_path")
    end
  end
end
