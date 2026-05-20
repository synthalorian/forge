class DashboardController < ApplicationController
  include AnvilHelper

  def show
    @forge_installed = forge_available?

    if @forge_installed
      load_stats
    else
      set_default_stats
    end
  rescue Forge::Database::NotFoundError, Forge::Database::CorruptedError
    @forge_installed = false
    set_default_stats
  end

  private

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue
    false
  end

  def load_stats
    stats = Forge::Statistics.new(database: forge_db)

    @backup_count = stats.backup_count
    @unique_repos = stats.unique_repos_count
    @disk_usage = stats.total_disk_usage
    @schedule_count = forge_db.schedule_count
    @average_backup_size = stats.average_backup_size
    @latest_backup = stats.latest_backup
    @top_repos = stats.top_repos(limit: 5)
    @weekly_trend = stats.weekly_trend
    @recent_activity = forge_db.backups(limit: 10)
  end

  def set_default_stats
    @backup_count = "—"
    @unique_repos = "—"
    @disk_usage = "—"
    @schedule_count = "—"
    @average_backup_size = "—"
    @latest_backup = nil
    @top_repos = []
    @weekly_trend = nil
  end

  def forge_db
    @forge_db ||= Forge::Database.new
  end
end
