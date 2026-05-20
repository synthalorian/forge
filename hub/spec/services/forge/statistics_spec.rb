require "rails_helper"

RSpec.describe Forge::Statistics do
  subject { described_class.new(database: database) }

  let(:database) { instance_double(Forge::Database) }

  describe "#backup_count" do
    it "returns the count from database" do
      allow(database).to receive(:backup_count).and_return(42)
      expect(subject.backup_count).to eq(42)
    end

    it "returns 0 when no backups exist" do
      allow(database).to receive(:backup_count).and_return(0)
      expect(subject.backup_count).to eq(0)
    end
  end

  describe "#unique_repos_count" do
    it "returns the count from database" do
      allow(database).to receive(:unique_repos).and_return(7)
      expect(subject.unique_repos_count).to eq(7)
    end

    it "returns 0 when no repos" do
      allow(database).to receive(:unique_repos).and_return(0)
      expect(subject.unique_repos_count).to eq(0)
    end
  end

  describe "#total_disk_usage" do
    it "returns the disk usage from database" do
      allow(database).to receive(:disk_usage).and_return(1_048_576)
      expect(subject.total_disk_usage).to eq(1_048_576)
    end

    it "returns 0 when no backups" do
      allow(database).to receive(:disk_usage).and_return(0)
      expect(subject.total_disk_usage).to eq(0)
    end
  end

  describe "#average_backup_size" do
    before do
      allow(database).to receive(:backup_count).and_return(count)
      allow(database).to receive(:disk_usage).and_return(total)
    end

    context "with backups" do
      let(:count) { 4 }
      let(:total) { 4_000_000 }

      it "returns the average size" do
        expect(subject.average_backup_size).to eq(1_000_000)
      end
    end

    context "with no backups" do
      let(:count) { 0 }
      let(:total) { 0 }

      it "returns 0" do
        expect(subject.average_backup_size).to eq(0)
      end
    end
  end

  describe "#latest_backup" do
    it "returns the most recent backup" do
      backup = { id: 1, repo_name: "my-repo", created_at: "2026-05-20T00:00:00Z", size_bytes: 1000 }
      allow(database).to receive(:backups).with(limit: 1).and_return([backup])
      expect(subject.latest_backup).to eq(backup)
    end

    it "returns nil when no backups" do
      allow(database).to receive(:backups).with(limit: 1).and_return([])
      expect(subject.latest_backup).to be_nil
    end
  end

  describe "#top_repos" do
    let(:backups) do
      [
        { repo_name: "alpha", size_bytes: 1000, created_at: "2026-01-01T00:00:00Z" },
        { repo_name: "alpha", size_bytes: 2000, created_at: "2026-01-02T00:00:00Z" },
        { repo_name: "alpha", size_bytes: 1500, created_at: "2026-01-03T00:00:00Z" },
        { repo_name: "beta", size_bytes: 3000, created_at: "2026-01-04T00:00:00Z" },
        { repo_name: "beta", size_bytes: 4000, created_at: "2026-01-05T00:00:00Z" },
        { repo_name: "gamma", size_bytes: 5000, created_at: "2026-01-06T00:00:00Z" },
      ]
    end

    it "returns repos sorted by backup count descending" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.top_repos(limit: 5)
      expect(result.map { |r| r[:name] }).to eq(%w[alpha beta gamma])
    end

    it "includes total size per repo" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.top_repos(limit: 5)
      alpha = result.find { |r| r[:name] == "alpha" }
      expect(alpha[:total_size]).to eq(4500)
      expect(alpha[:count]).to eq(3)
    end

    it "respects the limit parameter" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.top_repos(limit: 2)
      expect(result.size).to eq(2)
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return([])
      expect(subject.top_repos(limit: 5)).to eq([])
    end
  end

  describe "#backup_frequency" do
    it "returns weekly grouped counts" do
      backups = 12.times.map do |i|
        { repo_name: "repo", size_bytes: 1000, created_at: (Time.now - (i * 7).days).iso8601 }
      end
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.backup_frequency
      expect(result).to all(include(:week, :count))
      expect(result.size).to be <= 12
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return([])
      expect(subject.backup_frequency).to eq([])
    end
  end

  describe "#disk_usage_trend" do
    it "returns cumulative size over time" do
      backups = [
        { size_bytes: 1000, created_at: "2026-01-01T00:00:00Z" },
        { size_bytes: 2000, created_at: "2026-01-02T00:00:00Z" },
        { size_bytes: 3000, created_at: "2026-01-03T00:00:00Z" },
      ]
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.disk_usage_trend
      expect(result[0][:cumulative_size]).to eq(1000)
      expect(result[1][:cumulative_size]).to eq(3000)
      expect(result[2][:cumulative_size]).to eq(6000)
    end

    it "returns at most 12 entries" do
      backups = 20.times.map do |i|
        { size_bytes: 1000, created_at: (Time.now - (i * 1).days).iso8601 }
      end
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.disk_usage_trend
      expect(result.size).to eq(12)
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return([])
      expect(subject.disk_usage_trend).to eq([])
    end
  end

  describe "#weekly_trend" do
    it "returns up direction when this week has more backups" do
      now = Time.now
      backups = [
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 1.day).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 2.days).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 10.days).iso8601 },
      ]
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:up)
      expect(result[:current]).to eq(2)
      expect(result[:previous]).to eq(1)
    end

    it "returns down direction when last week had more backups" do
      now = Time.now
      backups = [
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 3.days).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 10.days).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 11.days).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 12.days).iso8601 },
      ]
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:down)
    end

    it "returns neutral when counts are equal" do
      now = Time.now
      backups = [
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 2.days).iso8601 },
        { repo_name: "repo", size_bytes: 1000, created_at: (now - 10.days).iso8601 },
      ]
      allow(database).to receive(:backups).with(limit: 10_000).and_return(backups)
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:neutral)
    end

    it "returns neutral when no backups" do
      allow(database).to receive(:backups).with(limit: 10_000).and_return([])
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:neutral)
      expect(result[:current]).to eq(0)
      expect(result[:previous]).to eq(0)
    end
  end
end
