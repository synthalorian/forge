require "rails_helper"

RSpec.describe "Flame Journal", type: :request do
  before do
    # Stub forge CLI calls to prevent hanging
    allow_any_instance_of(FlameController).to receive(:run_forge_command).and_return("stubbed forge output")
    # Stub forge availability so DB reads don't fail in test
    allow_any_instance_of(FlameController).to receive(:forge_available?).and_return(false)
  end

  describe "GET /flame/journal" do
    it "returns 200" do
      get "/flame/journal"
      expect(response).to have_http_status(:ok)
    end

    it "renders full HTML page (not turbo_stream)" do
      get "/flame/journal"
      expect(response.content_type).to include("text/html")
    end

    it "renders journal browser page" do
      get "/flame/journal"
      expect(response.body).to include("JOURNAL ENTRIES")
      expect(response.body).to include("AES-256-GCM")
      expect(response.body).to include("Search")
    end

    it "shows breadcrumb navigation" do
      get "/flame/journal"
      expect(response.body).to include("Flame")
      expect(response.body).to include("Journal")
    end

    it "shows back link to Flame" do
      get "/flame/journal"
      expect(response.body).to include("Back to Flame")
      expect(response.body).to include("/flame")
    end

    it "shows empty state when no entries" do
      get "/flame/journal"
      expect(response.body).to include("No journal entries found").or include("unavailable")
    end

    it "shows search form" do
      get "/flame/journal"
      expect(response.body).to include("/flame/journal/search")
      expect(response.body).to include("query")
    end
  end

  describe "GET /flame/journal/:id" do
    it "returns 200" do
      get "/flame/journal/1"
      expect(response).to have_http_status(:ok)
    end

    it "renders full HTML page" do
      get "/flame/journal/42"
      expect(response.content_type).to include("text/html")
    end

    it "shows entry ID" do
      get "/flame/journal/42"
      expect(response.body).to include("#42")
    end

    it "shows decrypted content area" do
      get "/flame/journal/1"
      expect(response.body).to include("Decrypted Content")
    end

    it "shows breadcrumb with entry ID" do
      get "/flame/journal/7"
      expect(response.body).to include("Flame")
      expect(response.body).to include("Journal")
      expect(response.body).to include("#7")
    end

    it "shows back link to journal" do
      get "/flame/journal/1"
      expect(response.body).to include("Back to Journal")
      expect(response.body).to include("/flame/journal")
    end

    it "displays forge reflect read output" do
      get "/flame/journal/1"
      expect(response.body).to include("stubbed forge output")
    end
  end

  describe "POST /flame/journal/search" do
    it "returns 200" do
      post "/flame/journal/search", params: { query: "prayer" }
      expect(response).to have_http_status(:ok)
    end

    it "renders full HTML page with search results" do
      post "/flame/journal/search", params: { query: "gratitude" }
      expect(response.content_type).to include("text/html")
    end

    it "shows search query in results" do
      post "/flame/journal/search", params: { query: "thanksgiving" }
      expect(response.body).to include("thanksgiving")
    end

    it "displays forge reflect search output" do
      post "/flame/journal/search", params: { query: "test query" }
      expect(response.body).to include("stubbed forge output")
    end

    it "shows breadcrumb with search context" do
      post "/flame/journal/search", params: { query: "hope" }
      expect(response.body).to include("Flame")
      expect(response.body).to include("Search")
    end

    it "handles empty query" do
      post "/flame/journal/search", params: { query: "" }
      expect(response).to have_http_status(:ok)
    end

    it "shows back link to journal" do
      post "/flame/journal/search", params: { query: "faith" }
      expect(response.body).to include("Back to Journal")
    end
  end
end
