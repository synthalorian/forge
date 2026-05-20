module Forge
  module Config
    def self.bin_path
      ENV.fetch("FORGE_BIN_PATH") do
        paths = %w[forge ~/.local/bin/forge /usr/local/bin/forge]
        paths.find { |p| File.executable?(File.expand_path(p)) } || "forge"
      end
    end

    def self.data_dir
      ENV.fetch("FORGE_DATA_DIR") do
        xdg_data = ENV.fetch("XDG_DATA_HOME") { File.expand_path("~/.local/share") }
        File.join(xdg_data, "forge")
      end
    end

    def self.db_path
      ENV.fetch("FORGE_DB_PATH") { File.join(data_dir, "forge.db") }
    end
  end
end

Rails.application.config.x.forge.bin_path = Forge::Config.bin_path
Rails.application.config.x.forge.data_dir = Forge::Config.data_dir
Rails.application.config.x.forge.db_path = Forge::Config.db_path

