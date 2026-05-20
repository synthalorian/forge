# frozen_string_literal: true

require "rails_helper"

RSpec.describe "Sample System Test", type: :system do
  it "renders the root page successfully" do
    visit root_path

    expect(page).to have_content("FORGE HUB")
    expect(page).to have_content("Command center for your forge infrastructure")
  end

  it "displays pillar navigation" do
    visit root_path

    expect(page).to have_content("Anvil")
    expect(page).to have_content("Bellows")
    expect(page).to have_content("Flame")
    expect(page).to have_content("Tongs")
    expect(page).to have_content("Crucible")
    expect(page).to have_content("Bridge")
  end
end
