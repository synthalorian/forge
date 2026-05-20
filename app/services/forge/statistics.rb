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
      backups = @database.backups(limit: 1)
      backups.first
    end

    def top_repos(limit: 5)
      all_backups = @database.backups(limit: 10_000)
      all_backups.group_by { |b| b[:repo_name] }
        .map { |name, backups| { name: name, count: backups.size, total_size: backups.sum { |b| b[:size_bytes] } } }
        .sort_by { |r| -r[:count] }
        .first(limit)
    end

    def backup_frequency
      all_backups = @database.backups(limit: 10_000)
      return [] if all_backups.empty?

      grouped = all_backups.group_by do |b|
        time = Time.parse(b[:created_at])
        time.strftime("%G-%V")
      end

      grouped.map { |week, backups| { week: week, count: backups.size } }
        .sort_by { |e| e[:week] }
        .last(12)
    end

    def disk_usage_trend
      all_backups = @database.backups(limit: 10_000)
      return [] if all_backups.empty?

      sorted = all_backups.sort_by { |b| b[:created_at] }
      cumulative = 0
      sorted.map do |b|
        cumulative += b[:size_bytes]
        { date: b[:created_at], cumulative_size: cumulative }
      end.last(12)
    end

    def weekly_trend
      all_backups = @database.backups(limit: 10_000)
      return { direction: :neutral, current: 0, previous: 0 } if all_backups.empty?

      now = Time.now
      this_week_start = (now - 7.days).beginning_of_day
      last_week_start = this_week_start - 7.days

      this_week = all_backups.count { |b| Time.parse(b[:created_at]) >= this_week_start }
      last_week = all_backups.count { |b| t = Time.parse(b[:created_at]); t >= last_week_start && t < this_week_start }

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
