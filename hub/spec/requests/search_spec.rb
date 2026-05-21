require "rails_helper"

RSpec.describe "Search", type: :request do
  def create_forge_db(backups: [], schedules: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["search_test_", ".db"])
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

  def create_agents_db(sessions: [])
    require "tempfile"
    require "sqlite3"
    temp = Tempfile.new(["agents_search_test_", ".db"])
    temp.close
    SQLite3::Database.open(temp.path) do |conn|
      conn.execute(<<-SQL)
        CREATE TABLE sessions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          agent_name TEXT NOT NULL,
          title TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
      SQL
      sessions.each do |s|
        conn.execute(
          "INSERT INTO sessions (agent_name, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
          [s[:agent_name], s[:title], s[:created_at], s[:updated_at] || s[:created_at]]
        )
      end
    end
    temp.path
  end

  around(:each) do |example|
    @saved_db = ENV["FORGE_DB_PATH"]
    example.run
  ensure
    ENV["FORGE_DB_PATH"] = @saved_db
  end

  describe "GET /search without query" do
    it "returns 200 with empty state" do
      get "/search"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Search the Forge")
    end
  end

  describe "GET /search?q=test" do
    it "returns 200 with query displayed" do
      allow_any_instance_of(SearchController).to receive(:search_all).and_return({})
      get "/search", params: { q: "test" }
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("test")
    end
  end

  describe "searching backups" do
    it "returns matching backups" do
      path = create_forge_db(backups: [
        { repo_path: "/repo/my-app", repo_name: "my-app",
          archive_path: "/arc/app.forge", sha256: "aa",
          size_bytes: 1_000_000, created_at: "2026-01-01T00:00:00Z" },
        { repo_path: "/repo/other-project", repo_name: "other-project",
          archive_path: "/arc/other.forge", sha256: "bb",
          size_bytes: 500_000, created_at: "2026-01-02T00:00:00Z" }
      ])
      ENV["FORGE_DB_PATH"] = path

      allow_any_instance_of(SearchController).to receive(:search_scripture).and_return([])
      allow_any_instance_of(SearchController).to receive(:search_dotfiles).and_return([])
      allow_any_instance_of(SearchController).to receive(:search_sessions).and_return([])

      get "/search", params: { q: "my-app" }
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("my-app")
      expect(response.body).not_to include("other-project")
    end
  end

  describe "searching sessions" do
    it "returns matching sessions from agents.db" do
      forge_path = create_forge_db(backups: [])
      agents_path = create_agents_db(sessions: [
        { agent_name: "opencode", title: "Fix authentication bug", created_at: "2026-01-01T00:00:00Z" },
        { agent_name: "codex", title: "Refactor parser", created_at: "2026-01-02T00:00:00Z" }
      ])

      ENV["FORGE_DB_PATH"] = forge_path

      allow_any_instance_of(SearchController).to receive(:agents_db_path).and_return(agents_path)
      allow_any_instance_of(SearchController).to receive(:search_scripture).and_return([])
      allow_any_instance_of(SearchController).to receive(:search_dotfiles).and_return([])

      get "/search", params: { q: "authentication" }
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("Fix authentication bug")
      expect(response.body).not_to include("Refactor parser")
    end
  end

  describe "search bar in topbar" do
    it "appears on the dashboard page" do
      ENV.delete("FORGE_DB_PATH")
      allow(File).to receive(:exist?).and_call_original
      allow(File).to receive(:exist?).with(/\/forge/).and_return(false)
      get "/"
      expect(response).to have_http_status(:ok)
      expect(response.body).to include('action="/search"')
      expect(response.body).to include('name="q"')
    end
  end

  describe "CLI calls are stubbed" do
    it "does not shell out to forge binary" do
      allow_any_instance_of(SearchController).to receive(:forge_available?).and_return(false)
      get "/search", params: { q: "anything" }
      expect(response).to have_http_status(:ok)
      expect(response.body).to include("No results found").or include("anything")
    end
  end
end
