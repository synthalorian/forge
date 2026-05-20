require "capybara/rspec"

Capybara.default_driver = :rack_test
Capybara.javascript_driver = :selenium_chrome_headless

RSpec.configure do |config|
  config.before(:each, type: :system) do
    driven_by :selenium, using: :headless_chrome, screen_size: [ 1400, 900 ]
  end

  config.after(:each, type: :system) do |example|
    if example.exception
      save_screenshot
      puts "\nScreenshot saved to tmp/screenshots/#{example.full_description.parameterize}.png"
    end
  end
end
