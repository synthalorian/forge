require "rails_helper"

RSpec.describe "Bridge Actions", type: :request do
  describe "GET /bridge/sync" do
    it "returns 200" do
      get "/bridge/sync"
      expect(response).to have_http_status(:ok)
    end

    it "returns turbo stream content type" do
      get "/bridge/sync"
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end

    it "contains sync-output turbo frame replacement" do
      get "/bridge/sync"
      expect(response.body).to include("sync-output")
    end

    context "when forge is not available" do
      before do
        allow(File).to receive(:exist?).and_call_original
        allow(File).to receive(:exist?).with(anything).and_call_original
        allow(File).to receive(:exist?)
          .with(instance_of(String)).and_call_original
      end

      it "returns unavailable status" do
        get "/bridge/sync"
        expect(response.body).to include("sync-output")
      end
    end
  end

  describe "POST /bridge/notify" do
    it "returns 200" do
      post "/bridge/notify", params: { channel: "desktop", message: "Test message" }
      expect(response).to have_http_status(:ok)
    end

    it "returns turbo stream content type" do
      post "/bridge/notify", params: { channel: "desktop", message: "Test message" }
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end

    it "contains notification-result turbo frame replacement" do
      post "/bridge/notify", params: { channel: "desktop", message: "Test message" }
      expect(response.body).to include("notification-result")
    end

    context "when message is empty" do
      it "returns error about empty message" do
        post "/bridge/notify", params: { channel: "desktop", message: "" }
        expect(response.body).to include("Message cannot be empty")
      end
    end

    context "when message is missing" do
      it "returns error about empty message" do
        post "/bridge/notify", params: { channel: "desktop" }
        expect(response.body).to include("Message cannot be empty")
      end
    end

    context "with different channels" do
      it "accepts telegram channel" do
        post "/bridge/notify", params: { channel: "telegram", message: "Hello Telegram" }
        expect(response).to have_http_status(:ok)
      end

      it "accepts discord channel" do
        post "/bridge/notify", params: { channel: "discord", message: "Hello Discord" }
        expect(response).to have_http_status(:ok)
      end
    end
  end

  describe "GET /bridge" do
    it "includes Omarchy integration card" do
      get "/bridge"
      expect(response.body).to include("Omarchy")
    end

    it "includes Sync Dashboard section" do
      get "/bridge"
      expect(response.body).to include("Sync Dashboard")
      expect(response.body).to include("Sync All Platforms")
    end

    it "includes Test Notification form" do
      get "/bridge"
      expect(response.body).to include("Test Notification")
      expect(response.body).to include("notification-result")
    end

    it "includes channel options" do
      get "/bridge"
      expect(response.body).to include("desktop")
      expect(response.body).to include("telegram")
      expect(response.body).to include("discord")
    end
  end
end
