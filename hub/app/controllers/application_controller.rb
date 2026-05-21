class ApplicationController < ActionController::Base
  # Only allow modern browsers supporting webp images, web push, badges, import maps, CSS nesting, and CSS :has.
  allow_browser versions: :modern

  # Changes to the importmap will invalidate the etag for HTML responses
  stale_when_importmap_changes

  # HTTP Basic Auth — enabled when FORGE_HUB_USERNAME is set in env
  before_action :authenticate_hub_if_configured

  private

  def authenticate_hub_if_configured
    return unless ENV["FORGE_HUB_USERNAME"].present?

    authenticate_or_request_with_http_basic("Forge Hub") do |username, password|
      # Use ActiveSupport::SecurityUtils to prevent timing attacks
      ActiveSupport::SecurityUtils.secure_compare(username, ENV["FORGE_HUB_USERNAME"].to_s) &
        ActiveSupport::SecurityUtils.secure_compare(password, ENV["FORGE_HUB_PASSWORD"].to_s)
    end
  end
end
