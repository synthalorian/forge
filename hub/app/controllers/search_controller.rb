class SearchController < ApplicationController
  include AnvilHelper

  def show
    @query = params[:q].to_s.strip
    @results = {}

    if @query.present?
      @results = search_all(@query)
    end

    @total_results = @results.values.sum(&:length)
  end

  private

  def search_all(query)
    {
      "Backups" => search_backups(query),
      "Schedules" => search_schedules(query),
      "Scripture" => search_scripture(query),
      "Sessions" => search_sessions(query),
      "Dotfiles" => search_dotfiles(query)
    }.select { |_, v| v.any? }
  end

  def search_backups(query)
    return [] unless forge_available?
    db = forge_db
    return [] unless db

    like = "%#{query}%"
    rows = Timeout.timeout(10) do
      db.backups(limit: 100).select { |b| b[:repo_name].to_s.downcase.include?(query.downcase) || b[:repo_path].to_s.downcase.include?(query.downcase) }
    end
    rows.first(10).map do |b|
      { title: b[:repo_name], subtitle: "#{human_size(b[:size_bytes])} — #{time_ago(b[:created_at])}", path: "/anvil/backups/#{b[:id]}", type: "Backup" }
    end
  rescue StandardError
    []
  end

  def search_schedules(query)
    return [] unless forge_available?
    db = forge_db
    return [] unless db

    like = "%#{query}%"
    Timeout.timeout(10) do
      db.schedules.select { |s| s[:target_path].to_s.downcase.include?(query.downcase) }.first(10).map do |s|
        { title: s[:target_path], subtitle: s[:cron_expression], path: "/anvil/schedules", type: "Schedule" }
      end
    end
  rescue StandardError
    []
  end

  def search_scripture(query)
    return [] unless forge_available?

    output = Timeout.timeout(30) do
      stdout, stderr, status = Open3.capture3("forge", "word", "search", query.to_s)
      status.success? ? stdout.strip : nil
    end
    return [] if output.nil? || output.empty?

    lines = output.split("\n").first(10)
    lines.map.with_index do |line, i|
      { title: line.truncate(100), subtitle: "Scripture match", path: "/flame", type: "Scripture" }
    end
  rescue StandardError
    []
  end

  def search_sessions(query)
    path = agents_db_path
    return [] unless path && File.exist?(path)

    like = "%#{query}%"
    Timeout.timeout(10) do
      db = SQLite3::Database.new(path, readonly: true, results_as_hash: true)
      rows = db.execute(
        "SELECT id, agent_name, title, created_at FROM sessions WHERE title LIKE ? OR agent_name LIKE ? LIMIT 10",
        [like, like]
      )
      db.close
      rows.map do |r|
        { title: r["title"].presence || "Session #{r['id']}", subtitle: "#{r['agent_name']} — #{time_ago(r['created_at'])}", path: "/bellows/sessions/#{r['id']}", type: "Session" }
      end
    end
  rescue StandardError
    []
  end

  def search_dotfiles(query)
    return [] unless forge_available?

    output = Timeout.timeout(10) do
      stdout, stderr, status = Open3.capture3("forge", "grip", "dotfiles", "list")
      status.success? ? stdout.strip : nil
    end
    return [] if output.nil? || output.empty?

    down_query = query.downcase
    output.split("\n").select { |l| l.downcase.include?(down_query) }.first(10).map do |line|
      parts = line.split(":", 2)
      name = parts.length == 2 ? parts[0].strip : line.strip
      { title: name, subtitle: parts.length == 2 ? parts[1].strip : line.strip, path: "/tongs/dotfiles", type: "Dotfile" }
    end
  rescue StandardError
    []
  end

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue StandardError
    false
  end

  def forge_db
    @forge_db ||= Forge::Database.new
  rescue StandardError
    nil
  end

  def agents_db_path
    File.join(File.dirname(Forge::Config.db_path), "db", "agents.db")
  rescue StandardError
    nil
  end
end
