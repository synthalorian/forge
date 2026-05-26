class Anvil::BackupsController < ApplicationController
  PAGE_SIZE = 25

  def index
    @page = (params[:page] || 1).to_i
    @page = 1 if @page < 1
    @offset = (@page - 1) * PAGE_SIZE

    @backups = forge_db.backups(limit: PAGE_SIZE, offset: @offset)
    @total_count = forge_db.backup_count
    @total_pages = [(@total_count.to_f / PAGE_SIZE).ceil, 1].max
    @disk_usage = forge_db.disk_usage
  rescue Forge::Database::NotFoundError
    render "anvil/no_forge"
  end

  def show
    @backup = forge_db.find_backup(params[:id])
    raise ActiveRecord::RecordNotFound unless @backup
  rescue Forge::Database::NotFoundError
    render "anvil/no_forge"
  end

  def trigger
    # Atomic check-and-set to prevent duplicate backup triggers
    cache_key = "forge_backup_running"
    unless Rails.cache.write(cache_key, true, unless_exist: true, expires_in: 5.minutes)
      redirect_to anvil_backups_path, alert: "A backup is already in progress."
      return
    end

    path = params[:path].to_s.strip
    if path.present?
      known_repos = forge_db.backups.map { |b| b[:repo_path] }
      unless known_repos.include?(path)
        Rails.cache.delete(cache_key)
        redirect_to anvil_backups_path, alert: "Unknown repository path."
        return
      end
    end

    begin
      job = BackupJob.perform_later(path: path.presence)
      session[:active_backup_job_id] = job&.job_id
      redirect_to anvil_backups_path, notice: "Backup started."
    rescue StandardError => e
      Rails.cache.delete(cache_key)
      redirect_to anvil_backups_path, alert: "Failed to start backup: #{e.message}"
    end
  end

  def browse
    @backup = forge_db.find_backup(params[:id])
    raise ActiveRecord::RecordNotFound unless @backup

    @archive_entries = list_archive_contents(@backup[:archive_path])
    @file_tree = build_file_tree(@archive_entries[:entries]) if @archive_entries[:entries].any?
  rescue Forge::Database::NotFoundError
    render "anvil/no_forge"
  end

  def chart_data
    backups = forge_db.backups(limit: 100)

    grouped = backups.group_by { |b| Date.parse(b[:created_at].to_s).iso8601 }
    thirty_days_ago = 30.days.ago.to_date
    chart_data = grouped.select { |date, _| date >= thirty_days_ago.iso8601 }.map do |date, group|
      {
        date: date,
        size: group.sum { |b| b[:size_bytes].to_i },
        repo_name: group.first[:repo_name],
        id: group.first[:id]
      }
    end

    render json: chart_data
  rescue Forge::Database::NotFoundError
    render json: []
  end

  def restore
    @backup = forge_db.find_backup(params[:id])
    unless @backup
      redirect_to anvil_backups_path, alert: "Backup not found."
      return
    end

    if Rails.cache.read("forge_restore_running_#{params[:id]}")
      redirect_to anvil_backup_path(params[:id]), alert: "Restore already in progress for this backup."
      return
    end

    RestoreJob.perform_later(backup_id: params[:id])
    redirect_to anvil_backup_path(params[:id]), notice: "Restore started. Files will be extracted to ./restored/#{@backup[:repo_name]}"
  rescue Forge::Database::NotFoundError
    render "anvil/no_forge"
  end

  private

  def forge_db
    @forge_db ||= Forge::Database.new
  end

  helper_method :backup_status

  def backup_status
    if Rails.cache.read("forge_backup_running")
      { running: true }
    else
      Rails.cache.read("forge_backup_result")
    end
  end

  def list_archive_contents(archive_path)
    return { entries: [], error: :archive_missing } unless archive_path && File.exist?(archive_path)

    require "open3"

    begin
      stdout, stderr, status = Open3.capture3(
        "zstd", "-d", "--stdout", archive_path, :stdin_data => ""
      )

      unless status.success?
        return { entries: [], error: :zstd_failed, message: stderr.strip }
      end

      tar_stdout, tar_stderr, tar_status = Open3.capture3(
        "tar", "-tvf", "-", :stdin_data => stdout
      )

      unless tar_status.success?
        return { entries: [], error: :tar_failed, message: tar_stderr.strip }
      end

      entries = tar_stdout.split("\n").reject(&:blank?).map do |line|
        # tar -tvf format:
        # drwxr-xr-x user/group       0 2024-01-01 12:00 path/to/dir/
        # -rw-r--r-- user/group    1234 2024-01-01 12:00 path/to/file
        parts = line.split(/\s+/)
        perms = parts[0] || ""
        size = parts[2].to_i
        # Path is everything after the date-time fields (5th split onward)
        full_path = parts[5..]&.join(" ") || ""
        full_path = full_path.chomp("/")

        is_dir = perms.start_with?("d") || line.end_with?("/")

        { path: full_path, directory: is_dir, size: size, permissions: perms }
      end

      { entries: entries, error: nil, total_count: entries.size }
    rescue Errno::ENOENT
      { entries: [], error: :zstd_not_installed }
    end
  end
end
