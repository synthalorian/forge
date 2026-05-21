class FlameController < ApplicationController
  include AnvilHelper

  def index
    @forge_available = forge_available?
    @daily_verse = fetch_daily_verse
    @journal_count = fetch_journal_count
    @journal_entries = fetch_journal_entries(limit: 5)
  rescue StandardError
    @forge_available = false
    @daily_verse = nil
    @journal_count = 0
    @journal_entries = []
  end

  def search_scripture
    query = params[:query].presence
    @search_results = run_forge_command("word", "search", query.to_s)
    render turbo_stream: turbo_stream.replace("flame-search-results",
      partial: "flame/command_output",
      locals: { output: @search_results, title: "Scripture Search" })
  end

  def journal
    @forge_available = forge_available?
    @page = [(params[:page] || 1).to_i, 1].max
    @per_page = 20
    @total_entries = fetch_journal_count
    @total_pages = (@total_entries / @per_page.to_f).ceil
    @entries = fetch_journal_entries_paginated(page: @page, per_page: @per_page)
  rescue StandardError
    @forge_available = false
    @entries = []
    @total_entries = 0
    @total_pages = 0
  end

  def journal_entry
    @entry_id = params[:id]
    @entry_content = run_forge_command("reflect", "read", @entry_id.to_s)
  rescue StandardError
    @entry_content = "Error loading entry"
  end

  def journal_search
    @query = params[:query].presence
    @forge_available = forge_available?
    @page = 1
    @per_page = 20
    @total_entries = 0
    @total_pages = 0
    @entries = []

    if @query
      @search_output = run_forge_command("reflect", "search", @query)
    else
      @search_output = "No query provided"
    end
  rescue StandardError
    @search_output = "Error during search"
  end

  def journal_entries
    @journal_entries = fetch_journal_entries(limit: 20)
    render turbo_stream: turbo_stream.replace("flame-journal-list",
      partial: "flame/journal_list",
      locals: { entries: @journal_entries })
  end

  def lookup_reference
    book = params[:book].presence
    chapter = params[:chapter].presence
    verse = params[:verse].presence

    reference = [book, chapter, verse].compact.join(" ")
    @reference_result = run_forge_command("word", "reference", reference)
    render turbo_stream: turbo_stream.replace("flame-reference-results",
      partial: "flame/command_output",
      locals: { output: @reference_result, title: "Reference Lookup" })
  end

  private

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue StandardError
    false
  end

  def fetch_daily_verse
    return nil unless forge_available?

    stdout, _, status = Open3.capture3("forge", "word", stdin_data: "")
    output = stdout.strip
    return nil if output.empty? || !status.success?
    output
  rescue StandardError => e
    Rails.logger.warn("fetch_daily_verse: #{e.message}")
    nil
  end

  def fetch_journal_count
    return 0 unless forge_available?
    spirit_db = spirit_db_path
    return 0 unless File.exist?(spirit_db)

    db = SQLite3::Database.new(spirit_db, readonly: true)
    begin
      db.get_first_value("SELECT COUNT(*) FROM journal_entries").to_i
    ensure
      db.close
    end
  rescue StandardError => e
    Rails.logger.warn("fetch_journal_count: #{e.message}")
    0
  end

  def fetch_journal_entries(limit: 20)
    return [] unless forge_available?
    spirit_db = spirit_db_path
    return [] unless File.exist?(spirit_db)

    db = SQLite3::Database.new(spirit_db, readonly: true)
    begin
      rows = db.execute("SELECT id, content, created_at FROM journal_entries ORDER BY created_at DESC LIMIT ?", limit)
      rows.map { |id, content, created_at| { id: id, content: content, created_at: created_at } }
    ensure
      db.close
    end
  rescue StandardError => e
    Rails.logger.warn("fetch_journal_entries: #{e.message}")
    []
  end

  def fetch_journal_entries_paginated(page: 1, per_page: 20)
    return [] unless forge_available?
    spirit_db = spirit_db_path
    return [] unless File.exist?(spirit_db)

    offset = (page - 1) * per_page
    db = SQLite3::Database.new(spirit_db, readonly: true)
    begin
      rows = db.execute(
        "SELECT id, content, created_at FROM journal_entries ORDER BY created_at DESC LIMIT ? OFFSET ?",
        [per_page, offset]
      )
      rows.map do |id, content, created_at|
        {
          id: id,
          content_size: content.respond_to?(:bytesize) ? content.bytesize : content.to_s.bytesize,
          created_at: created_at
        }
      end
    ensure
      db.close
    end
  rescue StandardError => e
    Rails.logger.warn("fetch_journal_entries_paginated: #{e.message}")
    []
  end

  def spirit_db_path
    File.join(File.dirname(Forge::Config.db_path), "db", "spirit.db")
  end

  def run_forge_command(*args)
    return "Forge not available" unless forge_available?
    Timeout.timeout(30) do
      stdout, stderr, status = Open3.capture3("forge", *args.map(&:to_s))
      status.success? ? stdout.strip : "Error: #{stderr.strip}"
    end
  rescue Timeout::Error
    "Command timed out after 30s"
  rescue StandardError => e
    "Error: #{e.message}"
  end
end
