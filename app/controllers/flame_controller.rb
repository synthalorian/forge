class FlameController < ApplicationController
  include AnvilHelper

  def index
    @forge_available = forge_available?
    @daily_verse = fetch_daily_verse
    @journal_count = fetch_journal_count
  rescue StandardError
    @forge_available = false
    @daily_verse = nil
    @journal_count = 0
  end

  private

  def fetch_daily_verse
    return nil unless @forge_available

    output = `forge word 2>/dev/null`.strip
    return nil if output.empty?
    output
  rescue StandardError
    nil
  end

  def fetch_journal_count
    return 0 unless @forge_available
    # Try to get count from spirit.db
    spirit_db = File.join(File.dirname(Forge::Config.db_path), "db", "spirit.db")
    return 0 unless File.exist?(spirit_db)

    db = SQLite3::Database.new(spirit_db, readonly: true)
    count = db.get_first_value("SELECT COUNT(*) FROM journal_entries").to_i
    db.close
    count
  rescue StandardError
    0
  end
end
