module Forge
  class Statistics
    def initialize(database: Forge::Database.new)
      @database = database
    end

    def backup_count
      @database.backup_count
    end

    def unique_repos_count
      @database.unique_repos
    end

    def total_disk_usage
      @database.disk_usage
    end

    def average_backup_size
      count = backup_count
      return 0 if count.zero?
      (total_disk_usage.to_f / count).round
    end

    def latest_backup
      @database.backups(limit: 1).first
    end

    def top_repos(limit: 5)
      @database.top_repos_by_count(limit: limit)
    end

    def backup_frequency
      @database.backup_frequency_weeks(limit: 12)
    end

    def disk_usage_trend
      @database.disk_usage_trend(limit: 12)
    end

    def weekly_trend
      counts = @database.weekly_backup_counts
      return { direction: :neutral, current: 0, previous: 0 } unless counts

      this_week = counts["this_week"].to_i
      last_week = counts["last_week"].to_i

      direction = if this_week > last_week
        :up
      elsif this_week < last_week
        :down
      else
        :neutral
      end

      { direction: direction, current: this_week, previous: last_week }
    end
  end
end
