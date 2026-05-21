require "rails_helper"

RSpec.describe "Bellows Actions", type: :request do
  before do
    # Stub forge CLI calls to prevent hanging on real forge binary
    allow_any_instance_of(BellowsController).to receive(:run_forge_command).and_return("stubbed output")
    allow_any_instance_of(BellowsController).to receive(:forge_available?).and_return(true)
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

    it "includes session management UI" do
      get "/bellows"
      expect(response.body).to include("Sessions")
      expect(response.body).to include("Quick Strike")
      expect(response.body).to include("Pipeline Runner")
    end
  end

  describe "GET /bellows/sessions" do
    it "returns 200" do
      get "/bellows/sessions"
      expect(response).to have_http_status(:ok)
    end
  end

  describe "GET /bellows/sessions/:id" do
    it "returns 200" do
      get "/bellows/sessions/1"
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /bellows/strike" do
    it "returns 200 with task param" do
      post "/bellows/strike", params: { task: "list files" }
      expect(response).to have_http_status(:ok)
    end

    it "returns bad request without task param" do
      post "/bellows/strike"
      expect(response).to have_http_status(:bad_request)
    end
  end

  describe "POST /bellows/pipeline" do
    it "returns 200 with toml param" do
      post "/bellows/pipeline", params: { toml: "[pipeline]\nname = \"test\"" }
      expect(response).to have_http_status(:ok)
    end

    it "returns bad request without toml param" do
      post "/bellows/pipeline"
      expect(response).to have_http_status(:bad_request)
    end
  end
end
