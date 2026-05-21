require "rails_helper"

RSpec.describe "Tongs Actions", type: :request do
  before do
    allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_return(
      "bashrc: ~/.bashrc"
    )
  end

  describe "GET /tongs/diagnose" do
    it "returns 200" do
      get "/tongs/diagnose"
      expect(response).to have_http_status(:ok)
    end

    it "returns turbo_stream content type" do
      get "/tongs/diagnose"
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end

    it "renders diagnose output" do
      get "/tongs/diagnose"
      expect(response.body).to include("tongs-output")
    end
  end

  describe "GET /tongs/dotfiles" do
    it "returns 200" do
      get "/tongs/dotfiles"
      expect(response).to have_http_status(:ok)
    end

    it "returns HTML content type (full page)" do
      get "/tongs/dotfiles"
      expect(response.media_type).to eq("text/html")
    end

    it "renders dotfiles page content" do
      get "/tongs/dotfiles"
      expect(response.body).to include("DOTFILES")
    end
  end

  describe "POST /tongs/dotfiles/track" do
    it "redirects to dotfiles page" do
      post "/tongs/dotfiles/track", params: { path: "~/.bashrc" }
      expect(response).to redirect_to("/tongs/dotfiles")
    end

    it "redirects with alert for empty path" do
      post "/tongs/dotfiles/track", params: { path: "" }
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to be_present
    end

    it "redirects with alert for missing path param" do
      post "/tongs/dotfiles/track"
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to be_present
    end
  end

  describe "GET /tongs/services" do
    it "returns 200" do
      get "/tongs/services"
      expect(response).to have_http_status(:ok)
    end

    it "returns turbo_stream content type" do
      get "/tongs/services"
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end

    it "renders services output" do
      get "/tongs/services"
      expect(response.body).to include("tongs-output")
    end
  end
end
