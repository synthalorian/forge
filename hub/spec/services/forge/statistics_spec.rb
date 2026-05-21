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
    let(:repos) do
      [
        { name: "alpha", count: 3, total_size: 4500 },
        { name: "beta", count: 2, total_size: 7000 },
        { name: "gamma", count: 1, total_size: 5000 },
      ]
    end

    before do
      allow(database).to receive(:top_repos_by_count).and_return(repos)
    end

    it "returns repos sorted by backup count descending" do
      result = subject.top_repos(limit: 5)
      expect(result.map { |r| r[:name] }).to eq(%w[alpha beta gamma])
    end

    it "includes total size per repo" do
      result = subject.top_repos(limit: 5)
      alpha = result.find { |r| r[:name] == "alpha" }
      expect(alpha[:total_size]).to eq(4500)
      expect(alpha[:count]).to eq(3)
    end

    it "respects the limit parameter" do
      allow(database).to receive(:top_repos_by_count).with(limit: 2).and_return(repos.first(2))
      result = subject.top_repos(limit: 2)
      expect(result.size).to eq(2)
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:top_repos_by_count).and_return([])
      expect(subject.top_repos(limit: 5)).to eq([])
    end
  end

  describe "#backup_frequency" do
    it "returns weekly grouped counts" do
      weeks = 6.times.map { |i| { week: "2026-W#{format('%02d', i + 1)}", count: (i + 1) * 2 } }
      allow(database).to receive(:backup_frequency_weeks).and_return(weeks)
      result = subject.backup_frequency
      expect(result).to all(include(:week, :count))
      expect(result.size).to be <= 12
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:backup_frequency_weeks).and_return([])
      expect(subject.backup_frequency).to eq([])
    end
  end

  describe "#disk_usage_trend" do
    it "returns cumulative size over time" do
      trend = [
        { date: "2026-01-01T00:00:00Z", cumulative_size: 1000 },
        { date: "2026-01-02T00:00:00Z", cumulative_size: 3000 },
        { date: "2026-01-03T00:00:00Z", cumulative_size: 6000 },
      ]
      allow(database).to receive(:disk_usage_trend).and_return(trend)
      result = subject.disk_usage_trend
      expect(result[0][:cumulative_size]).to eq(1000)
      expect(result[1][:cumulative_size]).to eq(3000)
      expect(result[2][:cumulative_size]).to eq(6000)
    end

    it "returns at most 12 entries" do
      trend = 12.times.map { |i| { date: "2026-01-#{format('%02d', i + 1)}T00:00:00Z", cumulative_size: (i + 1) * 1000 } }
      allow(database).to receive(:disk_usage_trend).and_return(trend)
      result = subject.disk_usage_trend
      expect(result.size).to eq(12)
    end

    it "returns empty array when no backups" do
      allow(database).to receive(:disk_usage_trend).and_return([])
      expect(subject.disk_usage_trend).to eq([])
    end
  end

  describe "#weekly_trend" do
    it "returns up direction when this week has more backups" do
      allow(database).to receive(:weekly_backup_counts).and_return({ "this_week" => 2, "last_week" => 1 })
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:up)
      expect(result[:current]).to eq(2)
      expect(result[:previous]).to eq(1)
    end

    it "returns down direction when last week had more backups" do
      allow(database).to receive(:weekly_backup_counts).and_return({ "this_week" => 1, "last_week" => 3 })
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:down)
    end

    it "returns neutral when counts are equal" do
      allow(database).to receive(:weekly_backup_counts).and_return({ "this_week" => 2, "last_week" => 2 })
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:neutral)
    end

    it "returns neutral when no backups" do
      allow(database).to receive(:weekly_backup_counts).and_return({ "this_week" => 0, "last_week" => 0 })
      result = subject.weekly_trend
      expect(result[:direction]).to eq(:neutral)
      expect(result[:current]).to eq(0)
      expect(result[:previous]).to eq(0)
    end
  end
end
