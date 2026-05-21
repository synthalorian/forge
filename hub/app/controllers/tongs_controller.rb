require "open3"

class TongsController < ApplicationController
  def index
    @system_info = gather_system_info
  rescue StandardError => e
    Rails.logger.error("TongsController#index: #{e.class}: #{e.message}")
    @system_info = {}
  end

  def diagnose
    @output = run_forge_grip("diagnose")
    render turbo_stream: turbo_stream.replace("tongs-output",
      partial: "tongs/command_output",
      locals: { output: @output, title: "Diagnose" })
  end

  def dotfiles
    raw = run_forge_argv(["grip", "dotfiles", "list"])
    @dotfiles = parse_dotfiles_output(raw)
  end

  def track_dotfile
    path = params[:path].to_s.strip
    if path.empty?
      redirect_to "/tongs/dotfiles", alert: "No path provided"
    else
      run_forge_argv(["grip", "dotfiles", "track", path])
      redirect_to "/tongs/dotfiles", notice: "Tracked: #{path}"
    end
  rescue Timeout::Error
    redirect_to "/tongs/dotfiles", alert: "Command timed out while tracking #{path}"
  end

  def restore_dotfile
    name = params[:name].to_s.strip
    if name.empty?
      redirect_to "/tongs/dotfiles", alert: "No dotfile name provided"
    else
      run_forge_argv(["grip", "dotfiles", "restore", name])
      redirect_to "/tongs/dotfiles", notice: "Restored: #{name}"
    end
  rescue Timeout::Error
    redirect_to "/tongs/dotfiles", alert: "Command timed out while restoring #{name}"
  end

  def services
    @output = run_systemd_services
    render turbo_stream: turbo_stream.replace("tongs-output",
      partial: "tongs/command_output",
      locals: { output: @output, title: "Running Services" })
  end

  private

  # ── Safe command execution — NO shell interpolation ────────

  # Run forge with argv array — immune to injection
  def run_forge_argv(args, timeout_seconds: 30)
    Timeout.timeout(timeout_seconds) do
      stdout, _stderr, status = Open3.capture3("forge", *args)
      status.success? ? stdout : ""
    end
  rescue Timeout::Error
    raise
  rescue StandardError => e
    Rails.logger.warn("run_forge_argv: #{e.message}")
    ""
  end

  def run_forge_grip(subcmd)
    output = run_forge_argv(["grip", subcmd])
    output.present? ? output : "Command returned no output (forge CLI may not be installed)"
  end

  def run_systemd_services
    stdout, _, status = Open3.capture3("systemctl", "list-units",
      "--type=service", "--state=running", "--no-pager", "--no-legend",
      stdin_data: "")
    return stdout if status.success? && stdout.present?

    # Fallback for non-systemd systems
    stdout2, _, _ = Open3.capture3("service", "--status-all", stdin_data: "")
    stdout2
  rescue StandardError
    ""
  end

  # ── System info gatherers (all use Open3) ──────────────────

  def gather_system_info
    {
      hostname: safe_argv("hostname").strip,
      kernel: safe_argv("uname", "-r").strip,
      os: parse_os_release,
      uptime: safe_argv("uptime", "-p").strip.gsub(/^up /, ""),
      cpu_info: parse_cpu_info,
      cpu_cores: safe_argv("nproc").strip,
      memory: parse_memory_info,
      memory_percent: parse_memory_percent,
      disk: parse_disk_info,
      disk_percent: parse_disk_percent,
      load_avg: File.read("/proc/loadavg").strip.split.first(3).join(", "),
      processes: parse_process_count,
      gpu_info: parse_gpu_info,
      temperatures: parse_temperatures
    }
  rescue StandardError => e
    Rails.logger.error("gather_system_info: #{e.message}")
    {}
  end

  # Safe argv-based command runner for non-sensitive system commands
  def safe_argv(*args)
    stdout, _, status = Open3.capture3(*args, stdin_data: "")
    status.success? ? stdout : ""
  rescue StandardError
    ""
  end

  def parse_os_release
    return "Unknown" unless File.exist?("/etc/os-release")
    File.readlines("/etc/os-release").each do |line|
      if line.start_with?("PRETTY_NAME=")
        return line.split("=", 2).last.to_s.strip.gsub(/"/, "")
      end
    end
    "Unknown"
  rescue StandardError
    "Unknown"
  end

  def parse_cpu_info
    stdout, _, _ = Open3.capture3("lscpu", stdin_data: "")
    stdout.each_line.find { |l| l.include?("Model name") }
      &.split(":", 2)&.last&.strip || "N/A"
  rescue StandardError
    "N/A"
  end

  def parse_memory_info
    stdout, _, _ = Open3.capture3("free", "-h", stdin_data: "")
    parts = stdout.split("\n").fetch(1, "").split
    return "N/A" if parts.length < 3
    "#{parts[2]} / #{parts[1]} used"
  rescue StandardError
    "N/A"
  end

  def parse_memory_percent
    stdout, _, _ = Open3.capture3("free", stdin_data: "")
    parts = stdout.split("\n").fetch(1, "").split
    return 0 if parts.length < 3
    used = parts[2].to_f
    total = parts[1].to_f
    return 0 if total.zero?
    ((used / total) * 100).round(1)
  rescue StandardError
    0
  end

  def parse_disk_info
    stdout, _, _ = Open3.capture3("df", "-h", "/", stdin_data: "")
    parts = stdout.split("\n").fetch(1, "").split
    return "N/A" if parts.length < 5
    "#{parts[2]} / #{parts[1]} (#{parts[4]})"
  rescue StandardError
    "N/A"
  end

  def parse_disk_percent
    stdout, _, _ = Open3.capture3("df", "/", stdin_data: "")
    parts = stdout.split("\n").fetch(1, "").split
    return 0 if parts.length < 5
    parts[4].to_s.gsub("%", "").to_f
  rescue StandardError
    0
  end

  def parse_process_count
    # Count entries in /proc instead of shelling out to ps aux | wc -l
    Dir.glob("/proc/[0-9]*").count.to_s
  rescue StandardError
    "N/A"
  end

  def parse_gpu_info
    stdout, _, status = Open3.capture3(
      "nvidia-smi", "--query-gpu=name,memory.total,memory.used",
      "--format=csv,noheader", stdin_data: ""
    )
    return stdout.strip if status.success? && stdout.strip.present?

    stdout2, _, _ = Open3.capture3("lspci", stdin_data: "")
    match = stdout2.each_line.find { |l| l.match?(/vga|3d|display/i) }
    match&.strip || "No GPU detected"
  rescue StandardError
    "No GPU detected"
  end

  def parse_temperatures
    stdout, _, status = Open3.capture3("sensors", stdin_data: "")
    return stdout.strip if status.success? && stdout.strip.present?

    # Fallback: read thermal zones from sysfs
    zones = Dir.glob("/sys/class/thermal/thermal_zone*/temp").map.with_index do |path, i|
      celsius = File.read(path).strip.to_f / 1000.0
      "Zone #{i}: #{celsius.round(1)}°C"
    rescue StandardError
      nil
    end.compact
    zones.empty? ? "N/A" : zones.join("\n")
  rescue StandardError
    "N/A"
  end

  def parse_dotfiles_output(raw)
    return [] if raw.blank?

    raw.split("\n").map do |line|
      parts = line.split(":", 2)
      if parts.length == 2
        { name: parts[0].strip, path: parts[1].strip, status: "tracked" }
      else
        { name: line.strip, path: line.strip, status: "tracked" }
      end
    end.select { |df| df[:name].present? }
  end
end
