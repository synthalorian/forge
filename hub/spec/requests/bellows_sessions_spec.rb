require "rails_helper"

RSpec.describe "Bellows Sessions", type: :request do
  before do
    allow_any_instance_of(BellowsController).to receive(:run_forge_command).and_return("stubbed output")
    allow_any_instance_of(BellowsController).to receive(:forge_available?).and_return(true)
    allow_any_instance_of(BellowsController).to receive(:agents_db_path).and_return("/tmp/test_agents.db")
  end

  describe "GET /bellows/sessions" do
    context "when agents.db has sessions" do
      before do
        db = SQLite3::Database.new("/tmp/test_agents.db")
        db.execute("CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY, agent_name TEXT, title TEXT, created_at TEXT, updated_at TEXT)")
        db.execute("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, session_id INTEGER, role TEXT, content TEXT, created_at TEXT)")
        db.execute("INSERT INTO sessions (id, agent_name, title, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                   [1, "opencode", "Test Session 1", "2026-05-20 10:00:00", "2026-05-20 10:30:00"])
        db.execute("INSERT INTO sessions (id, agent_name, title, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                   [2, "codex", "Another Session", "2026-05-19 14:00:00", "2026-05-19 15:00:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [1, 1, "user", "Hello", "2026-05-20 10:00:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [2, 1, "assistant", "Hi there", "2026-05-20 10:01:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [3, 2, "user", "Task please", "2026-05-19 14:00:00"])
        db.close
      end

      after do
        File.delete("/tmp/test_agents.db") if File.exist?("/tmp/test_agents.db")
      end

      it "returns 200" do
        get "/bellows/sessions"
        expect(response).to have_http_status(:ok)
      end

      it "renders sessions list page with breadcrumb" do
        get "/bellows/sessions"
        expect(response.body).to include("SESSIONS")
        expect(response.body).to include("Bellows")
        expect(response.body).to include("Sessions")
      end

      it "displays session cards with agent name and title" do
        get "/bellows/sessions"
        expect(response.body).to include("opencode")
        expect(response.body).to include("Test Session 1")
        expect(response.body).to include("codex")
        expect(response.body).to include("Another Session")
      end

      it "shows message counts" do
        get "/bellows/sessions"
        expect(response.body).to include("2 messages")
        expect(response.body).to include("1 messages")
      end

      it "includes links to session detail pages" do
        get "/bellows/sessions"
        expect(response.body).to include('/bellows/sessions/1')
        expect(response.body).to include('/bellows/sessions/2')
      end

      it "includes delete buttons for each session" do
        get "/bellows/sessions"
        expect(response.body).to include("Delete")
      end
    end

    context "when agents.db does not exist" do
      before do
        File.delete("/tmp/test_agents.db") if File.exist?("/tmp/test_agents.db")
      end

      it "returns 200 with empty state" do
        get "/bellows/sessions"
        expect(response).to have_http_status(:ok)
        expect(response.body).to include("No sessions found")
      end
    end
  end

  describe "GET /bellows/sessions/:id" do
    context "when session exists" do
      before do
        db = SQLite3::Database.new("/tmp/test_agents.db")
        db.execute("CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY, agent_name TEXT, title TEXT, created_at TEXT, updated_at TEXT)")
        db.execute("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, session_id INTEGER, role TEXT, content TEXT, created_at TEXT)")
        db.execute("INSERT INTO sessions (id, agent_name, title, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                   [42, "opencode", "Detail Test Session", "2026-05-20 10:00:00", "2026-05-20 10:30:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [1, 42, "user", "What is the answer?", "2026-05-20 10:00:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [2, 42, "assistant", "42 is the answer to everything", "2026-05-20 10:01:00"])
        db.execute("INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
                   [3, 42, "system", "Session initialized", "2026-05-20 09:59:00"])
        db.close
      end

      after do
        File.delete("/tmp/test_agents.db") if File.exist?("/tmp/test_agents.db")
      end

      it "returns 200" do
        get "/bellows/sessions/42"
        expect(response).to have_http_status(:ok)
      end

      it "renders session detail page with breadcrumb" do
        get "/bellows/sessions/42"
        expect(response.body).to include("Detail Test Session")
        expect(response.body).to include("Bellows")
        expect(response.body).to include("Sessions")
      end

      it "displays session metadata" do
        get "/bellows/sessions/42"
        expect(response.body).to include("opencode")
        expect(response.body).to include("2026-05-20 10:00:00")
      end

      it "renders chat-like message timeline" do
        get "/bellows/sessions/42"
        expect(response.body).to include("YOU")
        expect(response.body).to include("ASSISTANT")
        expect(response.body).to include("SYSTEM")
        expect(response.body).to include("What is the answer?")
        expect(response.body).to include("42 is the answer to everything")
        expect(response.body).to include("Session initialized")
      end

      it "applies role-specific styling classes" do
        get "/bellows/sessions/42"
        expect(response.body).to include("bg-neon-purple/10")
        expect(response.body).to include("bg-neon-cyan/10")
        expect(response.body).to include("bg-bg-surface/30")
      end

      it "shows back link to sessions list" do
        get "/bellows/sessions/42"
        expect(response.body).to include("Back to Sessions")
      end
    end

    context "when session does not exist" do
      before do
        db = SQLite3::Database.new("/tmp/test_agents.db")
        db.execute("CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY, agent_name TEXT, title TEXT, created_at TEXT, updated_at TEXT)")
        db.execute("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, session_id INTEGER, role TEXT, content TEXT, created_at TEXT)")
        db.close
      end

      after do
        File.delete("/tmp/test_agents.db") if File.exist?("/tmp/test_agents.db")
      end

      it "returns 200 with session not found message" do
        get "/bellows/sessions/999"
        expect(response).to have_http_status(:ok)
        expect(response.body).to include("Session not found")
      end
    end

    context "when agents.db does not exist" do
      before do
        File.delete("/tmp/test_agents.db") if File.exist?("/tmp/test_agents.db")
      end

      it "returns 200 with session not found" do
        get "/bellows/sessions/1"
        expect(response).to have_http_status(:ok)
        expect(response.body).to include("Session not found")
      end
    end
  end

  describe "DELETE /bellows/sessions/:id" do
    it "redirects to sessions list" do
      delete "/bellows/sessions/1"
      expect(response).to redirect_to("/bellows/sessions")
    end

    it "displays flash notice after deletion" do
      delete "/bellows/sessions/42"
      follow_redirect!
      expect(response.body).to include("Session 42 deleted")
    end
  end
end
