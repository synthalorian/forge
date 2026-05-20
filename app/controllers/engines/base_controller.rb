class Engines::BaseController < ApplicationController
  layout false

  private

  def render_coming_soon(pillar_name:, description:, features:, version_requirement:, tagline: "")
    render "engines/coming_soon", locals: {
      pillar_name: pillar_name,
      tagline: tagline,
      description: description,
      features: features,
      version_requirement: version_requirement
    }
  end
end