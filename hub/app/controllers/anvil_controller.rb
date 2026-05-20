class AnvilController < ApplicationController
  def index
    redirect_to anvil_backups_path
  end
end