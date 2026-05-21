class BellowsController < ApplicationController
  include AnvilHelper

  def index
    @agents = detect_agents
    @forge_available = forge_available?
  rescue StandardError
    @agents = []
    @forge_available = false
  end

  def sessions
    @page = (params[:page] || 1).to_i
    @page = 1 if @page < 1
    per_page = 20
    offset = (@page - 1) * per_page

    db = agents_db_connection
    if db
      begin
        @sessions = db.execute(
          "SELECT s.id, s.agent_name, s.title, s.created_at, s.updated_at, " \
          "(SELECT COUNT(*) FROM messages WHERE session_id = s.id) AS message_count " \
          "FROM sessions s ORDER BY s.updated_at DESC LIMIT ? OFFSET ?",
          [per_page, offset]
        )
        total = db.get_first_value("SELECT COUNT(*) FROM sessions").to_i
        @total_pages = (total.to_f / per_page).ceil
      ensure
        db.close
      end
    else
      @sessions = []
      @total_pages = 1
    end
  end

  def session_detail
    @session_id = params[:id]

    db = agents_db_connection
    if db
      begin
        @session = db.get_first_row(
          "SELECT id, agent_name, title, created_at, updated_at FROM sessions WHERE id = ?",
          [@session_id]
        )
        @messages = db.execute(
          "SELECT id, role, content, created_at FROM messages WHERE session_id = ? ORDER BY created_at ASC",
          [@session_id]
        )
      ensure
        db.close
      end
    else
      @session = nil
      @messages = []
    end
  end

  def delete_session
    id = params[:id].to_s.strip
    unless id.match?(/\A[a-zA-Z0-9_-]+\z/)
      redirect_to "/bellows/sessions", alert: "Invalid session ID."
      return
    end
    run_forge_command(["breathe", "sessions", "delete", id])
    redirect_to "/bellows/sessions", notice: "Session #{id} deleted"
  end

  def run_pipeline
    toml = params[:toml].presence
    return render plain: "No pipeline definition provided", status: :bad_request unless toml

    tmpfile = Tempfile.new(%w[pipeline .toml])
    begin
      tmpfile.write(toml)
      tmpfile.flush
      @output = run_forge_command(["breathe", "pipe", tmpfile.path])
    ensure
      tmpfile.close!
    end

    render turbo_stream: turbo_stream.replace("bellows-output",
      partial: "bellows/command_output",
      locals: { output: @output, title: "Pipeline Output" })
  end

  def strike
    task = params[:task].presence
    return render plain: "No task provided", status: :bad_request unless task

    @output = run_forge_command(["strike", task])
    render turbo_stream: turbo_stream.replace("bellows-output",
      partial: "bellows/command_output",
      locals: { output: @output, title: "Strike Result" })
  end

  private

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue StandardError
    false
  end

  def run_forge_command(args, timeout_seconds: 30)
    return "Forge not available" unless forge_available?
    bin = find_forge_binary
    return "Forge binary not found" unless bin
    Timeout.timeout(timeout_seconds) do
      stdout, stderr, status = Open3.capture3(bin, *args)
      status.success? ? stdout : "Error: #{stderr}"
    end
  rescue Timeout::Error
    "Command timed out after #{timeout_seconds}s"
  rescue StandardError => e
    "Error: #{e.message}"
  end

  def find_forge_binary
    Forge::Client.new(path: nil).bin_path
  rescue StandardError
    nil
  end

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

  def agents_db_path
    File.join(File.dirname(Forge::Config.db_path), "db", "agents.db")
  end

  def agents_db_connection
    path = agents_db_path
    return nil unless File.exist?(path)
    require "sqlite3"
    db = SQLite3::Database.new(path, results_as_hash: true)
    if block_given?
      begin
        yield db
      ensure
        db.close
      end
    else
      db
    end
  rescue StandardError
    nil
  end

  def check_agent_status(name)
    case name
    when "opencode"
      binary = system("which", "opencode", out: File::NULL, err: File::NULL)
      running = system("pgrep", "-x", "opencode", out: File::NULL, err: File::NULL)
      { status: running ? "running" : (binary ? "stopped" : "not_installed"),
        model: "glm-5.1 (Z.AI)" }
    when "llama-swap"
      config_exists = File.exist?(ENV["LLAMA_SWAP_CONFIG"] || File.expand_path("~/llama.cpp/llama-swap/config.yaml"))
      running = system("pgrep", "-f", "llama", out: File::NULL, err: File::NULL)
      { status: running ? "running" : (config_exists ? "stopped" : "not_installed"),
        model: config_exists ? "Local models" : nil }
    when "hermes"
      binary = system("which", "hermes", out: File::NULL, err: File::NULL)
      { status: binary ? "stopped" : "not_installed",
        model: nil }
    when "codex"
      binary = system("which", "codex", out: File::NULL, err: File::NULL)
      { status: binary ? "stopped" : "not_installed",
        model: nil }
    else
      { status: "not_installed", model: nil }
    end
  end
end
