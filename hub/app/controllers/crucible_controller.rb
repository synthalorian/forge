class CrucibleController < ApplicationController
  include AnvilHelper

  def index
    @forge_available = forge_available?
    return unless @forge_available

    @chord_examples = generate_chord_examples
    @palette_example = generate_palette_example
  rescue StandardError
    @forge_available = false
    @chord_examples = []
    @palette_example = nil
  end

  def chords
    key = params[:key].presence || "C"
    scale = params[:scale].presence || "major"
    mood = params[:mood].presence

    @output = run_forge_melt_chords(key, scale, mood)
    render turbo_stream: turbo_stream.replace("crucible-chords-output",
      partial: "crucible/command_output",
      locals: { output: @output, title: "Chord Progression" })
  end

  def palette
    color = params[:color].presence
    harmony = params[:harmony].presence || "complementary"
    format = params[:format].presence || "terminal"
    file = params[:file].presence

    if file
      @output = run_forge_melt_palette_from_image(file, format)
    else
      @output = run_forge_melt_palette(color, harmony, format)
    end
    render turbo_stream: turbo_stream.replace("crucible-palette-output",
      partial: "crucible/command_output",
      locals: { output: @output, title: "Color Palette" })
  end

  def upload_palette_image
    uploaded = params[:image]
    format = params[:format].presence || "terminal"

    unless uploaded
      @output = "No image file provided"
      render turbo_stream: turbo_stream.replace("crucible-palette-output",
        partial: "crucible/command_output",
        locals: { output: @output, title: "Color Palette" })
      return
    end

    tempfile = nil
    begin
      # Save uploaded file to a tempfile
      ext = File.extname(uploaded.original_filename).presence || ".png"
      tempfile = Tempfile.new(["palette_upload", ext])
      tempfile.binmode
      tempfile.write(uploaded.read)
      tempfile.rewind
      tempfile.close

      @output = run_forge_melt_palette_from_image(tempfile.path, format)
    rescue StandardError => e
      @output = "Error: #{e.message}"
    ensure
      tempfile&.unlink
    end

    render turbo_stream: turbo_stream.replace("crucible-palette-output",
      partial: "crucible/command_output",
      locals: { output: @output || "Error processing image", title: "Color Palette" })
  end

  def diagram
    diag_type = params[:type].presence || "flow"
    description = params[:description].presence

    @output = run_forge_melt_diagram(diag_type, description)
    render turbo_stream: turbo_stream.replace("crucible-diagram-output",
      partial: "crucible/command_output",
      locals: { output: @output, title: "Diagram" })
  end

  private

  def forge_available?
    File.exist?(Forge::Config.db_path)
  rescue StandardError
    false
  end

  def run_forge_melt_chords(key, scale, mood)
    args = ["melt", "chords", key]
    args += ["--scale", scale] if scale.present?
    args += ["--mood", mood] if mood.present?
    run_forge_command(args)
  end

  def run_forge_melt_palette(color, harmony, format)
    args = ["melt", "palette"]
    args << color if color.present?
    args += ["--harmony", harmony] if harmony.present?
    args += ["--format", format] if format.present?
    run_forge_command(args)
  end

  def run_forge_melt_palette_from_image(file, format)
    args = ["melt", "palette", "--file", file]
    args += ["--format", format] if format.present? && format != "terminal"
    run_forge_command(args)
  end

  def run_forge_melt_diagram(diag_type, description)
    args = ["melt", "diagram", diag_type]
    args += ["--description", description] if description.present?
    run_forge_command(args)
  end

  def run_forge_command(args)
    return "Forge not available" unless forge_available?
    bin = find_forge_binary
    return "Forge binary not found" unless bin
    stdout, stderr, status = Open3.capture3(bin, *args)
    status.success? ? stdout : "Error: #{stderr}"
  rescue StandardError => e
    "Error: #{e.message}"
  end

  def find_forge_binary
    Forge::Client.new(path: nil).bin_path
  rescue StandardError
    nil
  end

  def generate_chord_examples
    [
      { key: "C", scale: "major", label: "C Major" },
      { key: "Am", scale: "minor", label: "A Minor" },
      { key: "G", scale: "mixolydian", label: "G Mixolydian" },
      { key: "D", scale: "dorian", label: "D Dorian" }
    ]
  end

  def generate_palette_example
    run_forge_melt_palette("#FF6B9D", "complementary", "terminal")
  end
end
