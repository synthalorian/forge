require "rails_helper"

RSpec.describe "Pillar Routes", type: :request do
  describe "GET /anvil" do
    it "redirects to /anvil/backups" do
      get "/anvil"
      expect(response).to redirect_to("/anvil/backups")
    end
  end

  describe "GET /anvil/backups" do
    it "returns 200" do
      get "/anvil/backups"
      expect(response).to have_http_status(:ok)
    end

    it "renders backup page" do
      get "/anvil/backups"
      # May render the backup list or the no-forge setup page
      expect(response.body).to include("BACKUPS").or include("forge")
    end
  end

  describe "GET /bellows" do
    it "returns 200" do
      get "/bellows"
      expect(response).to have_http_status(:ok)
    end

    it "renders Bellows page with agent status" do
      get "/bellows"
      expect(response.body).to include("BELLOWS")
      expect(response.body).to include("opencode")
      expect(response.body).to include("llama-swap")
      expect(response.body).to include("forge breathe")
      expect(response.body).to include("forge strike")
    end
  end

  describe "GET /flame" do
    it "returns 200" do
      get "/flame"
      expect(response).to have_http_status(:ok)
    end

    it "renders Flame page with scripture features" do
      get "/flame"
      expect(response.body).to include("FLAME")
      expect(response.body).to include("31,103")
      expect(response.body).to include("forge word")
      expect(response.body).to include("forge reflect")
      expect(response.body).to include("forge rest")
    end
  end

  describe "GET /tongs" do
    it "returns 200" do
      get "/tongs"
      expect(response).to have_http_status(:ok)
    end

    it "renders Tongs page with system info" do
      get "/tongs"
      expect(response.body).to include("TONGS")
      expect(response.body).to include("System Info")
      expect(response.body).to include("Resource Usage")
    end
  end

  describe "GET /crucible" do
    it "returns 200" do
      get "/crucible"
      expect(response).to have_http_status(:ok)
    end

    it "renders Crucible Coming Soon page" do
      get "/crucible"
      expect(response.body).to include("CRUCIBLE")
      expect(response.body).to include("forge melt")
    end
  end

  describe "GET /bridge" do
    it "returns 200" do
      get "/bridge"
      expect(response).to have_http_status(:ok)
    end

    it "renders Bridge Coming Soon page" do
      get "/bridge"
      expect(response.body).to include("BRIDGE")
      expect(response.body).to include("forge bridge")
    end
  end
end
