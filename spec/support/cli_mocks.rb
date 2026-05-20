# frozen_string_literal: true

require "open3"

module CliMocks
  FORGE_BINARY = File.expand_path("~/.local/bin/forge").freeze
  FORGE_VERSION = "1.0.0".freeze

  def self.forge_list_response(backups: [])
    {
      backups: backups.map do |b|
        {
          id: b[:id] || 1,
          repo_name: b[:repo_name] || "test-repo",
          repo_path: b[:repo_path] || "/home/user/projects/test-repo",
          archive_path: b[:archive_path] || "/tmp/forge/archives/test.forge",
          size_bytes: b[:size_bytes] || 1_048_576,
          backup_type: b[:backup_type] || "full",
          created_at: b[:created_at] || Time.now.utc.iso8601
        }
      end
    }.to_json
  end

  def self.forge_version_response
    FORGE_VERSION
  end

  def self.forge_status_response
    "healthy"
  end

  def setup_forge_cli_mocks(overrides: {})
    @cli_mocks = {
      version: overrides[:version] || forge_version_response,
      "list --json" => overrides.fetch("list --json", forge_list_response),
      status: overrides[:status] || forge_status_response,
      "backup" => overrides.fetch("backup", "")
    }

    allow(File).to receive(:exist?).with(FORGE_BINARY).and_return(true)

    allow(Open3).to receive(:capture3).and_wrap_original do |method, *args|
      cmd = args[0] || args[1]
      mock_response_for(cmd)
    end
  end

  def mock_response_for(cmd)
    @cli_mocks.each do |key, response|
      next unless cmd.to_s.include?(key.to_s)

      status = double(success?: true, exitstatus: 0)
      return [ response, "", status ]
    end
    [ "", "Unknown command: #{cmd}", double(success?: false, exitstatus: 1) ]
  end

  def stub_forge_binary_not_found
    allow(File).to receive(:exist?).with(FORGE_BINARY).and_return(false)
    allow(Open3).to receive(:capture3).with(/forge/).and_raise(
      Errno::ENOENT, "No such file or directory - forge"
    )
  end

  def stub_forge_command_failed(exit_code: 1, stderr: "Command failed")
    status = double(success?: false, exitstatus: exit_code)
    allow(Open3).to receive(:capture3).and_return([ "", stderr, status ])
  end
end

RSpec.configure do |config|
  config.include CliMocks
end
