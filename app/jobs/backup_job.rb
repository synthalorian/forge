require "open3"

class BackupJob < ApplicationJob
  queue_as :default

  def perform(path: nil, job_id: self.job_id)
    if Rails.cache.read("forge_backup_running")
      raise "Backup already in progress"
    end

    Rails.cache.write("forge_backup_running", true, expires_in: 10.minutes)
    broadcast_status(job_id, "running", "Starting backup...")

    binary = Forge::Client.new.bin_path
    cmd = [binary]
    cmd += path ? ["quench", path] : ["quench"]

    Open3.popen3(*cmd) do |stdin, stdout, stderr, wait_thr|
      stdin.close

      stdout.each_line do |line|
        broadcast_output(job_id, line.chomp)
      end

      stderr.each_line do |line|
        broadcast_output(job_id, "STDERR: #{line.chomp}")
      end

      exit_status = wait_thr.value
      if exit_status.success?
        broadcast_status(job_id, "success", "Backup complete!")
        Rails.cache.write("forge_backup_result", { "status" => "success", "message" => "Backup complete!" })
      else
        broadcast_status(job_id, "error", "Backup failed with exit code #{exit_status.exitstatus}")
        Rails.cache.write("forge_backup_result", { "status" => "error", "message" => "Backup failed with exit code #{exit_status.exitstatus}" })
      end
    end
  rescue => e
    broadcast_status(job_id, "error", e.message)
    Rails.cache.write("forge_backup_result", { "status" => "error", "message" => e.message })
  ensure
    Rails.cache.delete("forge_backup_running")
  end

  private

  def broadcast_output(job_id, line)
    Turbo::StreamsChannel.broadcast_append_to(
      "backup_progress_#{job_id}",
      target: "backup-output",
      html: "<div class=\"font-mono text-sm text-text-muted\">#{ERB::Util.html_escape(line)}</div>"
    )
  end

  def broadcast_status(job_id, status, message)
    color = case status
            when "running" then "text-neon-yellow"
            when "success" then "text-neon-green"
            when "error" then "text-neon-red"
            end
    Turbo::StreamsChannel.broadcast_replace_to(
      "backup_progress_#{job_id}",
      target: "backup-status",
      html: "<span id=\"backup-status\" class=\"#{color} font-mono text-sm\">#{ERB::Util.html_escape(message)}</span>"
    )
  end
end
