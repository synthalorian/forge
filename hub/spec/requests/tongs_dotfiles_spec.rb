require "rails_helper"

RSpec.describe "Tongs Dotfiles", type: :request do
  before do
    allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_return(
      "bashrc: ~/.bashrc\nvimrc: ~/.vimrc\nhyprland: ~/.config/hypr/hyprland.conf"
    )
  end

  describe "GET /tongs/dotfiles" do
    it "returns 200" do
      get "/tongs/dotfiles"
      expect(response).to have_http_status(:ok)
    end

    it "renders HTML (not turbo_stream)" do
      get "/tongs/dotfiles"
      expect(response.media_type).to eq("text/html")
    end

    it "renders the dotfiles page title" do
      get "/tongs/dotfiles"
      expect(response.body).to include("DOTFILES")
    end

    it "shows breadcrumb with Tongs link" do
      get "/tongs/dotfiles"
      expect(response.body).to include("Tongs")
      expect(response.body).to include("/tongs")
      expect(response.body).to include("Dotfiles")
    end

    it "lists tracked dotfiles" do
      get "/tongs/dotfiles"
      expect(response.body).to include("bashrc")
      expect(response.body).to include("~/.bashrc")
      expect(response.body).to include("vimrc")
      expect(response.body).to include("~/.vimrc")
    end

    it "shows restore button for each dotfile" do
      get "/tongs/dotfiles"
      expect(response.body).to include("Restore")
    end

    it "shows the track form" do
      get "/tongs/dotfiles"
      expect(response.body).to include("Track a Dotfile")
      expect(response.body).to include('name="path"')
    end

    it "shows back link to tongs" do
      get "/tongs/dotfiles"
      expect(response.body).to include("Back to Tongs")
    end
  end

  describe "POST /tongs/dotfiles/track" do
    it "redirects to dotfiles page" do
      post "/tongs/dotfiles/track", params: { path: "~/.bashrc" }
      expect(response).to redirect_to("/tongs/dotfiles")
    end

    it "sets notice on success" do
      post "/tongs/dotfiles/track", params: { path: "~/.bashrc" }
      expect(flash[:notice]).to include("~/.bashrc")
    end

    it "calls run_forge_argv with track args" do
      allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_return("tracked")
      expect_any_instance_of(TongsController).to receive(:run_forge_argv).with(["grip", "dotfiles", "track", "~/.bashrc"])

      post "/tongs/dotfiles/track", params: { path: "~/.bashrc" }
    end

    it "redirects with alert when path is empty" do
      post "/tongs/dotfiles/track", params: { path: "" }
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to include("No path provided")
    end

    it "redirects with alert when path is missing" do
      post "/tongs/dotfiles/track"
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to include("No path provided")
    end

    it "handles timeout gracefully" do
      allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_raise(Timeout::Error)

      post "/tongs/dotfiles/track", params: { path: "~/.bashrc" }
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to include("timed out")
    end
  end

  describe "POST /tongs/dotfiles/restore" do
    it "redirects to dotfiles page" do
      post "/tongs/dotfiles/restore", params: { name: "bashrc" }
      expect(response).to redirect_to("/tongs/dotfiles")
    end

    it "sets notice on success" do
      post "/tongs/dotfiles/restore", params: { name: "bashrc" }
      expect(flash[:notice]).to include("bashrc")
    end

    it "calls run_forge_argv with restore args" do
      allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_return("restored")
      expect_any_instance_of(TongsController).to receive(:run_forge_argv).with(["grip", "dotfiles", "restore", "bashrc"])

      post "/tongs/dotfiles/restore", params: { name: "bashrc" }
    end

    it "redirects with alert when name is empty" do
      post "/tongs/dotfiles/restore", params: { name: "" }
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to include("No dotfile name provided")
    end

    it "handles timeout gracefully" do
      allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_raise(Timeout::Error)

      post "/tongs/dotfiles/restore", params: { name: "bashrc" }
      expect(response).to redirect_to("/tongs/dotfiles")
      expect(flash[:alert]).to include("timed out")
    end
  end

  describe "dotfiles page with empty output" do
    before do
      allow_any_instance_of(TongsController).to receive(:run_forge_argv).and_return("")
    end

    it "shows empty state message" do
      get "/tongs/dotfiles"
      expect(response.body).to include("No dotfiles tracked yet")
    end
  end
end
