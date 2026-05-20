# frozen_string_literal: true

require "rails_helper"
require "open3"

RSpec.describe Forge::Client do
  # Use a path that can never exist on this system to avoid
  # interference from the actual forge binary in PATH or ~/.local/bin
  let(:test_binary) { "/opt/forge/test/bin/forge" }
  let(:client) { described_class.new(path: test_binary) }

  before do
    allow(File).to receive(:executable?).and_call_original
    allow(File).to receive(:executable?).with(test_binary).and_return(true)
  end

  def stub_forge_default_paths_not_found
    Forge::Client::DEFAULT_PATHS.each do |p|
      expanded = File.expand_path(p)
      allow(File).to receive(:executable?).with(expanded).and_return(false)
    end
  end

  describe "#initialize" do
    it "raises BinaryNotFoundError when binary does not exist" do
      original_path = ENV["PATH"]
      ENV["PATH"] = "/dev/null"
      stub_forge_default_paths_not_found

      expect { described_class.new(path: "/nonexistent/forge") }
        .to raise_error(Forge::Client::BinaryNotFoundError, /not found/)
    ensure
      ENV["PATH"] = original_path
    end
  end

  describe "#version" do
    it "returns the version string from forge --version" do
      expected_version = "forge 0.2.0\n"
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "--version")
        .and_return([expected_version, "", status])

      expect(client.version).to eq("forge 0.2.0")
    end
  end

  describe "#installed?" do
    it "returns true when binary is executable" do
      expect(client.installed?).to be true
    end
  end

  describe "#list_backups" do
    it "passes --json flag and returns parsed result" do
      json_response = { "backups" => [{ "id" => 1, "repo_name" => "test" }] }.to_json
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "list", "--json")
        .and_return([json_response, "", status])

      result = client.list_backups
      expect(result).to be_a(Hash)
      expect(result["backups"].first["repo_name"]).to eq("test")
    end
  end

  describe "#backup" do
    it "runs quench command without path" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "quench")
        .and_return(["Backup complete", "", status])

      result = client.backup
      expect(result[:stdout]).to eq("Backup complete")
      expect(result[:exit_code]).to eq(0)
    end

    it "runs quench command with path" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "quench", "/my/project")
        .and_return(["Backup complete", "", status])

      client.backup(path: "/my/project")
    end
  end

  describe "#restore" do
    it "runs restore with backup id" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "restore", "42")
        .and_return(["Restored successfully", "", status])

      client.restore(42)
    end
  end

  describe "#status" do
    it "returns status output" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "status")
        .and_return(["healthy\n", "", status])

      expect(client.status).to eq("healthy\n")
    end
  end

  describe "#schedule_list" do
    it "runs schedule list command" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "schedule", "list")
        .and_return(["Schedule list output", "", status])

      expect(client.schedule_list).to eq("Schedule list output")
    end
  end

  describe "#schedule_add" do
    it "runs schedule add with cron and path" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "schedule", "add", "0 2 * * *", "/my/project")
        .and_return(["Schedule created", "", status])

      client.schedule_add("0 2 * * *", "/my/project")
    end
  end

  describe "#schedule_remove" do
    it "runs schedule remove with id" do
      status = instance_double(Process::Status, exitstatus: 0)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "schedule", "remove", "5")
        .and_return(["Schedule removed", "", status])

      client.schedule_remove(5)
    end
  end

  describe "error handling" do
    it "raises CommandError on non-zero exit code" do
      status = instance_double(Process::Status, exitstatus: 1)

      expect(Open3).to receive(:capture3)
        .with(test_binary, "list", "--json")
        .and_return(["", "error: something broke", status])

      expect { client.list_backups }
        .to raise_error(Forge::Client::CommandError, /exit 1/)
    end

    it "raises TimeoutError when command times out" do
      expect(Open3).to receive(:capture3)
        .with(test_binary, "status")
        .and_raise(Timeout::Error)

      expect { client.status }
        .to raise_error(Forge::Client::TimeoutError, /timed out/)
    end
  end
end
