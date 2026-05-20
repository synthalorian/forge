class RestoreJob < ApplicationJob
  queue_as :default

  def perform(backup_id:)
    Rails.cache.write("forge_restore_running_#{backup_id}", true, expires_in: 5.minutes)
    client = Forge::Client.new
    result = client.restore(backup_id)
    Rails.cache.write("forge_restore_result_#{backup_id}", {
      "status" => "success",
      "output" => result[:stdout]
    }, expires_in: 5.minutes)
  rescue Forge::Client::CommandError => e
    Rails.cache.write("forge_restore_result_#{backup_id}", {
      "status" => "error",
      "message" => e.message
    }, expires_in: 5.minutes)
  rescue => e
    Rails.cache.write("forge_restore_result_#{backup_id}", {
      "status" => "error",
      "message" => "Unexpected error: #{e.message}"
    }, expires_in:5.minutes)
  ensure
    Rails.cache.delete("forge_restore_running_#{backup_id}")
  end
end
