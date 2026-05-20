class BridgeController < ApplicationController
  include AnvilHelper

  def index
    @forge_available = forge_available?
    return unless @forge_available

    @integrations = detect_integrations
    @hooks = detect_hooks
  rescue StandardError
    @forge_available = false
    @integrations = []
    @hooks = []
  end

  private

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
      { name: "llama-swap", available: File.exist?("/home/synth/llama.cpp/llama-swap/config.yaml"),
        type: "agent", icon: "🦙",
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
        hint: "Start with bin/rails server" }
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
    system("which #{cmd} >/dev/null 2>&1")
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
end
