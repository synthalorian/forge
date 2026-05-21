require "rails_helper"

RSpec.describe "Flame Actions", type: :request do
  describe "GET /flame" do
    it "returns 200" do
      get "/flame"
      expect(response).to have_http_status(:ok)
    end

    it "renders Flame page with interactive scripture search" do
      get "/flame"
      expect(response.body).to include("FLAME")
      expect(response.body).to include("Scripture Search")
      expect(response.body).to include("Reference Lookup")
      expect(response.body).to include("Recent Journal Entries")
      expect(response.body).to include("flame-search-results")
      expect(response.body).to include("flame-reference-results")
      expect(response.body).to include("flame-journal-list")
    end

    it "preserves existing content" do
      get "/flame"
      expect(response.body).to include("31,103")
      expect(response.body).to include("forge word")
      expect(response.body).to include("forge reflect")
      expect(response.body).to include("forge rest")
      expect(response.body).to include("Sabbath Mode")
      expect(response.body).to include("Today's Verse")
    end
  end

  describe "POST /flame/search" do
    it "returns turbo stream response" do
      post "/flame/search", params: { query: "love" }
      expect(response).to have_http_status(:ok)
    end

    it "renders search results in turbo frame" do
      post "/flame/search", params: { query: "grace" }
      expect(response.content_type).to include("text/vnd.turbo-stream.html")
    end

    it "handles empty query" do
      post "/flame/search", params: { query: "" }
      expect(response).to have_http_status(:ok)
    end
  end

  describe "GET /flame/journal" do
    it "returns 200" do
      get "/flame/journal"
      expect(response).to have_http_status(:ok)
    end

    it "renders journal browser as full HTML page" do
      get "/flame/journal"
      expect(response.content_type).to include("text/html")
      expect(response.body).to include("JOURNAL ENTRIES")
    end
  end

  describe "POST /flame/reference" do
    it "returns turbo stream response" do
      post "/flame/reference", params: { book: "John", chapter: "3", verse: "16" }
      expect(response).to have_http_status(:ok)
    end

    it "renders reference results in turbo frame" do
      post "/flame/reference", params: { book: "Psalm", chapter: "23", verse: "1" }
      expect(response.content_type).to include("text/vnd.turbo-stream.html")
    end

    it "handles missing parameters" do
      post "/flame/reference", params: { book: "", chapter: "", verse: "" }
      expect(response).to have_http_status(:ok)
    end
  end
end
