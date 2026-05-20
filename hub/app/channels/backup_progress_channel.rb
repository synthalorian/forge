class BackupProgressChannel < ApplicationCable::Channel
  def subscribed
    stream_from "backup_progress_#{params[:job_id]}"
  end

  def unsubscribed
    # Cleanup
  end
end
