class BellowsController < ApplicationController
  include AnvilHelper

  def index
    @agents = detect_agents
    @forge_available = forge_available?
  rescue StandardError
    @agents = []
    @forge_available = false
  end

  private

  def detect_agents
    agents = [
      { name: "opencode", type: "local", icon: "⚡" },
      { name: "llama-swap", type: "local", icon: "🦙" },
      { name: "hermes", type: "remote", icon: "使者" },
      { name: "codex", type: "cli", icon: "⚡" }
    ]

    agents.map do |agent|
      status = check_agent_status(agent[:name])
      agent.merge(status)
    end
  end

  def check_agent_status(name)
    case name
    when "opencode"
      binary = system("which opencode >/dev/null 2>&1")
      running = system("pgrep -x opencode >/dev/null 2>&1")
      { status: running ? "running" : (binary ? "stopped" : "not_installed"),
        model: "glm-5.1 (Z.AI)" }
    when "llama-swap"
      config_exists = File.exist?("/home/synth/llama.cpp/llama-swap/config.yaml")
      running = system("pgrep -f llama >/dev/null 2>&1")
      { status: running ? "running" : (config_exists ? "stopped" : "not_installed"),
        model: config_exists ? "Local models" : nil }
    when "hermes"
      binary = system("which hermes >/dev/null 2>&1")
      { status: binary ? "stopped" : "not_installed",
        model: nil }
    when "codex"
      binary = system("which codex >/dev/null 2>&1")
      { status: binary ? "stopped" : "not_installed",
        model: nil }
    else
      { status: "not_installed", model: nil }
    end
  end
end
