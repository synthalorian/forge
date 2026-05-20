class TongsController < ApplicationController
  def index
    @system_info = gather_system_info
  rescue StandardError
    @system_info = {}
  end

  private

  def gather_system_info
    {
      hostname: safe_command("hostname").strip,
      kernel: safe_command("uname -r").strip,
      os: safe_command("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2").strip,
      uptime: safe_command("uptime -p 2>/dev/null || uptime").strip.gsub(/^up /, ""),
      cpu_info: safe_command("lscpu 2>/dev/null | grep 'Model name' | cut -d: -f2").strip,
      cpu_cores: safe_command("nproc").strip,
      memory: parse_memory_info,
      disk: parse_disk_info,
      load_avg: safe_command("cat /proc/loadavg 2>/dev/null").strip.split.first(3).join(", "),
      processes: safe_command("ps aux 2>/dev/null | wc -l").strip
    }
  end

  def safe_command(cmd)
    `#{cmd} 2>/dev/null`.to_s
  rescue StandardError
    ""
  end

  def parse_memory_info
    output = safe_command("free -h 2>/dev/null")
    return "N/A" if output.empty?

    lines = output.split("\n")
    return "N/A" if lines.length < 2

    parts = lines[1].split
    return "N/A" if parts.length < 3

    "#{parts[2]} / #{parts[1]} used"
  end

  def parse_disk_info
    output = safe_command("df -h / 2>/dev/null")
    return "N/A" if output.empty?

    lines = output.split("\n")
    return "N/A" if lines.length < 2

    parts = lines[1].split
    return "N/A" if parts.length < 5

    "#{parts[2]} / #{parts[1]} (#{parts[4]})"
  end
end
