class Anvil::SchedulesController < ApplicationController
  def index
    @schedules = forge_db.schedules
  rescue Forge::Database::NotFoundError
    render "anvil/no_forge"
  end

  def create
    cron = params[:cron_expression]
    path = params[:target_path]

    if cron.blank? || path.blank?
      redirect_to anvil_schedules_path, alert: "Cron expression and target path are required."
      return
    end

    Forge::Client.new.schedule_add(cron, path)
    redirect_to anvil_schedules_path, notice: "Schedule added."
  rescue Forge::Client::CommandError => e
    redirect_to anvil_schedules_path, alert: "Failed to add schedule: #{e.message}"
  end

  def destroy
    Forge::Client.new.schedule_remove(params[:id])
    redirect_to anvil_schedules_path, notice: "Schedule removed."
  rescue Forge::Client::CommandError => e
    redirect_to anvil_schedules_path, alert: "Failed to remove schedule: #{e.message}"
  end

  def toggle
    redirect_to anvil_schedules_path, notice: "Schedule toggling requires forge CLI v0.2+."
  end

  private

  def forge_db
    @forge_db ||= Forge::Database.new
  end
end
