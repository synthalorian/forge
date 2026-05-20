require "open3"

module Forge
  class Client
    class BinaryNotFoundError < StandardError; end
    class CommandError < StandardError; end
    class TimeoutError < StandardError; end

    DEFAULT_PATHS = %w[forge ~/.local/bin/forge /usr/local/bin/forge].freeze
    QUERY_TIMEOUT = 60
    MUTATION_TIMEOUT = 300

    def initialize(path: nil)
      @bin_path = resolve_binary(path || Forge::Config.bin_path)
    end

    attr_reader :bin_path

    def version
      run!("--version", timeout: QUERY_TIMEOUT)[:stdout].strip
    end

    def installed?
      @bin_path != "forge" || find_in_path
    end

    def list_backups(json: true)
      args = json ? %w[list --json] : %w[list]
      result = run!(*args, timeout: QUERY_TIMEOUT)
      json ? JSON.parse(result[:stdout]) : result[:stdout]
    end

    def backup(path: nil)
      args = path ? ["quench", path] : ["quench"]
      run!(*args, timeout: MUTATION_TIMEOUT)
    end

    def restore(id)
      run!("restore", id.to_s, timeout: MUTATION_TIMEOUT)
    end

    def status
      run!("status", timeout: QUERY_TIMEOUT)[:stdout]
    end

    def schedule_list
      run!("schedule", "list", timeout: QUERY_TIMEOUT)[:stdout]
    end

    def schedule_add(cron, path)
      run!("schedule", "add", cron, path, timeout: QUERY_TIMEOUT)
    end

    def schedule_remove(id)
      run!("schedule", "remove", id.to_s, timeout: QUERY_TIMEOUT)
    end

    private

    def resolve_binary(path)
      return path if path && File.executable?(path)
      return path if path && find_in_path(path)
      return DEFAULT_PATHS[0] if find_in_path(DEFAULT_PATHS[0])
      DEFAULT_PATHS.each do |p|
        expanded = File.expand_path(p)
        return expanded if File.executable?(expanded)
      end
      raise BinaryNotFoundError, "Forge binary not found. Install it from https://github.com/synthalorian/forge"
    end

    def find_in_path(cmd = nil)
      target = cmd || @bin_path
      ENV["PATH"].split(File::PATH_SEPARATOR).any? do |dir|
        File.executable?(File.join(dir, target))
      end
    end

    def run!(*args, timeout:)
      stdout, stderr, status = nil
      cmd = [@bin_path, *args.map(&:to_s)]

      begin
        Timeout.timeout(timeout) do
          stdout, stderr, status = Open3.capture3(*cmd)
        end
      rescue Timeout::Error
        raise TimeoutError, "Forge command timed out after #{timeout}s: #{cmd.join(" ")}"
      end

      stdout = sanitize_output(stdout)
      stderr = sanitize_output(stderr)
      exit_code = status.exitstatus

      if exit_code == 134
        raise CommandError, "Forge process crashed (signal 6). Please check your forge installation."
      elsif exit_code != 0
        msg = stderr.empty? ? stdout : stderr
        raise CommandError, "Forge command failed (exit #{exit_code}): #{msg.strip}"
      end

      { stdout: stdout, stderr: stderr, exit_code: exit_code }
    end

    def sanitize_output(text)
      return "" if text.nil?
      text.encode("UTF-8", "UTF-8", invalid: :replace, undef: :replace, replace: "")
    end
  end
end
