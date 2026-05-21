require "open3"

class BridgeController < ApplicationController
  include AnvilHelper

  before_action :set_forge_status, only: [ :index, :sync ]

  def index
    return unless @forge_available

    @integrations = detect_integrations
    @hooks = detect_hooks
    @omarchy = detect_omarchy
  rescue StandardError => e
    Rails.logger.error("BridgeController#index: #{e.class}: #{e.message}")
    @forge_available = false
    @integrations = []
    @hooks = []
    @omarchy = nil
  end

  def sync
    unless @forge_available
      @sync_output = "Forge CLI not available"
      @sync_status = :unavailable
      render turbo_stream: turbo_stream.replace("sync-output",
        partial: "bridge/sync_result",
        locals: { output: @sync_output, status: @sync_status })
      return
    end

    stdout, stderr, status = run_forge_command([ "bridge", "sync", "--verbose" ])
    @sync_output = status ? stdout : "Error: #{stderr}"
    @sync_status = status ? :success : :error

    render turbo_stream: turbo_stream.replace("sync-output",
      partial: "bridge/sync_result",
      locals: { output: @sync_output, status: @sync_status })
  end

  def send_notification
    channel = params[:channel].presence || "desktop"
    message = params[:message].presence

    unless message.present?
      @notification_result = "Message cannot be empty"
      @notification_status = :error
      render turbo_stream: turbo_stream.replace("notification-result",
        partial: "bridge/notification_result",
        locals: { result: @notification_result, status: @notification_status })
      return
    end

    unless forge_available?
      @notification_result = "Forge CLI not available"
      @notification_status = :error
      render turbo_stream: turbo_stream.replace("notification-result",
        partial: "bridge/notification_result",
        locals: { result: @notification_result, status: @notification_status })
      return
    end

    stdout, stderr, status = run_forge_command([ "bridge", "notify", "-c", channel, message ])
    @notification_result = status ? stdout : "Error: #{stderr}"
    @notification_status = status ? :success : :error

    render turbo_stream: turbo_stream.replace("notification-result",
      partial: "bridge/notification_result",
      locals: { result: @notification_result, status: @notification_status })
  end

  private

  def set_forge_status
    @forge_available = forge_available?
  end

  def detect_integrations
    [
      { name: "Forge CLI", available: true, type: "core", icon: "🔨",
        hint: "Running now" },
      { name: "Git", available: which?("git"), type: "vcs", icon: "⎇",
        hint: which?("git") ? nil : "Install git" },
      { name: "zstd", available: which?("zstd"), type: "compress", icon: "▤",
        hint: which?("zstd") ? nil : "Install zstd" },
      { name: "OpenCode", available: which?("opencode"), type: "agent", icon: "⚡",
        hint: which?("opencode") ? nil : "Install opencode" },
      { name: "llama-swap", available: File.exist?(Forge::Config.llama_swap_config_path),
        type: "agent", icon: "\U0001f999",
        hint: "Local model server" },
      { name: "Hermes", available: which?("hermes"), type: "agent", icon: "使者",
        hint: which?("hermes") ? nil : "Install hermes-agent" },
      { name: "Codex CLI", available: which?("codex"), type: "cli", icon: "⚡",
        hint: which?("codex") ? nil : "Install codex" },
      { name: "ripgrep", available: which?("rg"), type: "search", icon: "🔍",
        hint: which?("rg") ? nil : "Install ripgrep" },
      { name: "Docker", available: which?("docker"), type: "container", icon: "🐋",
        hint: which?("docker") ? nil : "Install docker" },
      { name: "Redis", available: which?("redis-cli"), type: "cache", icon: "⚡",
        hint: which?("redis-cli") ? nil : "Install redis" },
      { name: "Forge Hub", available: check_localhost_3000, type: "web", icon: "◈",
        hint: "Start with bin/rails server" },
      { name: "Omarchy", available: File.directory?(File.expand_path("~/.config/hypr")),
        type: "desktop", icon: "🏔",
        hint: File.directory?(File.expand_path("~/.config/hypr")) ? "Hyprland detected" : "No Hyprland config" }
    ]
  end

  def detect_hooks
    hooks_dir = File.join(Forge::Config.data_dir, "scripts")
    return [] unless File.directory?(hooks_dir)

    Dir.entries(hooks_dir).filter_map do |entry|
      next if entry.start_with?(".")
      path = File.join(hooks_dir, entry)
      { name: entry, executable: File.executable?(path) }
    end.sort_by { |h| h[:name] }
  end

  def which?(cmd)
    system("which", cmd, out: File::NULL, err: File::NULL)
  end

  def check_localhost_3000
    system("curl -s -o /dev/null -w '%{http_code}' --max-time 2 http://localhost:3000 2>/dev/null | grep -qE '^[23]'")
  rescue StandardError
    false
  end

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue StandardError
    false
  end

  def run_forge_command(args, timeout_seconds: 30)
    bin = find_forge_binary
    return [ "", "Forge binary not found", false ] unless bin
    Timeout.timeout(timeout_seconds) do
      stdout, stderr, status = Open3.capture3(bin, *args)
      [ stdout, stderr, status.success? ]
    end
  rescue Timeout::Error
    [ "", "Command timed out after #{timeout_seconds}s", false ]
  rescue StandardError => e
    [ "", e.message, false ]
  end

  def find_forge_binary
    Forge::Client.new(path: nil).bin_path
  rescue StandardError
    nil
  end

  def detect_omarchy
    {
      omarchy_bin: which?("omarchy"),
      hyprland: File.directory?(File.expand_path("~/.config/hypr")),
      waybar: File.directory?(File.expand_path("~/.config/waybar"))
    }
  end
end
