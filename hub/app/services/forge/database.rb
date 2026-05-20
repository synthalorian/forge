require "sqlite3"

module Forge
  class Database
    class NotFoundError < StandardError; end
    class CorruptedError < StandardError; end
    class BusyError < StandardError; end

    RETRY_ATTEMPTS = 3
    RETRY_DELAYS = [0.1, 0.2, 0.4].freeze

    def initialize(path: nil)
      @path = path || ENV.fetch("FORGE_DB_PATH") { default_db_path }
      validate!
    end

    def backups(limit: 50, offset: 0)
      with_retry do |db|
        db.execute(
          "SELECT id, repo_path, repo_name, archive_path, sha256, size_bytes,
                  branch_count, tag_count, commit_count, backup_type, created_at
           FROM backups ORDER BY created_at DESC LIMIT ? OFFSET ?",
          [limit, offset]
        ).map { |row| row_to_backup(row) }
      end
    end

    def backup_count
      with_retry { |db| db.get_first_value("SELECT COUNT(*) FROM backups") || 0 }
    end

    def find_backup(id)
      with_retry do |db|
        row = db.get_first_row(
          "SELECT id, repo_path, repo_name, archive_path, sha256, size_bytes,
                  branch_count, tag_count, commit_count, backup_type, created_at
           FROM backups WHERE id = ?",
          [id.to_i]
        )
        row ? row_to_backup(row) : nil
      end
    end

    def schedules
      with_retry do |db|
        db.execute(
          "SELECT id, cron_expression, target_path, enabled, last_run, created_at
           FROM schedules ORDER BY id"
        ).map { |row| row_to_schedule(row) }
      end
    end

    def disk_usage
      with_retry do |db|
        db.get_first_value("SELECT COALESCE(SUM(size_bytes), 0) FROM backups") || 0
      end
    end

    def unique_repos
      with_retry do |db|
        db.get_first_value("SELECT COUNT(DISTINCT repo_name) FROM backups") || 0
      end
    end

    def schedule_count
      with_retry do |db|
        db.get_first_value("SELECT COUNT(*) FROM schedules") || 0
      end
    end

    private

    def validate!
      raise NotFoundError, "Forge database not found at #{@path}" unless File.exist?(@path)
    end

    def default_db_path
      xdg = ENV.fetch("XDG_DATA_HOME") { File.expand_path("~/.local/share") }
      File.join(xdg, "forge", "forge.db")
    end

    def with_retry(&block)
      last_error = nil
      RETRY_ATTEMPTS.times do |attempt|
        begin
          db = create_connection
          return block.call(db)
        rescue SQLite3::BusyException => e
          last_error = e
          sleep RETRY_DELAYS[attempt] if attempt < RETRY_ATTEMPTS - 1
        rescue SQLite3::NotADatabaseException => e
          raise CorruptedError, "Forge database at #{@path} appears corrupted: #{e.message}"
        end
      end
      raise BusyError, "Forge database is busy after #{RETRY_ATTEMPTS} retries: #{last_error&.message}"
    end

    def create_connection
      db = SQLite3::Database.new(@path, readonly: true, results_as_hash: true)
      db.busy_timeout = 5000
      db
    end

    def row_to_backup(row)
      {
        id: row["id"],
        repo_path: row["repo_path"],
        repo_name: row["repo_name"],
        archive_path: row["archive_path"],
        sha256: row["sha256"],
        size_bytes: row["size_bytes"],
        branch_count: row["branch_count"],
        tag_count: row["tag_count"],
        commit_count: row["commit_count"],
        backup_type: row["backup_type"],
        created_at: row["created_at"]
      }
    end

    def row_to_schedule(row)
      {
        id: row["id"],
        cron_expression: row["cron_expression"],
        target_path: row["target_path"],
        enabled: row["enabled"] == 1,
        last_run: row["last_run"],
        created_at: row["created_at"]
      }
    end
  end
end
