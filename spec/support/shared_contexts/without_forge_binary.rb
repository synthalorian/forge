# frozen_string_literal: true

require "open3"

RSpec.shared_context "without_forge_binary", without_forge_binary: true do
  let(:forge_binary_path) { File.expand_path("~/.local/bin/forge") }

  around(:each) do |example|
    original_exist = File.method(:exist?)

    allow(File).to receive(:exist?).with(forge_binary_path) { false }

    example.run
  ensure
  end
end
