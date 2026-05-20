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
    if Rails.cache.read("forge_backup_running")
      redirect_to anvil_backups_path, alert: "A backup is already in progress."
      return
    end

    job = BackupJob.perform_later(path: params[:path])
    session[:active_backup_job_id] = job&.job_id
    redirect_to anvil_backups_path, notice: "Backup started."
  end

  def browse
    @backup = forge_db.find_backup(params[:id])
    raise ActiveRecord::RecordNotFound unless @backup

    @archive_entries = list_archive_contents(@backup[:archive_path])
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
        "tar", "-tf", "-", :stdin_data => stdout
      )

      unless tar_status.success?
        return { entries: [], error: :tar_failed, message: tar_stderr.strip }
      end

      entries = tar_stdout.split("\n").reject(&:blank?).map do |line|
        path = line.chomp("/")
        is_dir = line.end_with?("/")
        { path: path, directory: is_dir }
      end

      { entries: entries, error: nil, total_count: entries.size }
    rescue Errno::ENOENT
      { entries: [], error: :zstd_not_installed }
    end
  end
end
