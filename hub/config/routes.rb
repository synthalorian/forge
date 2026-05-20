Rails.application.routes.draw do
  get "up" => "rails/health#show", as: :rails_health_check

  # Anvil engine — fully wired
  get "/anvil" => "anvil#index"
  namespace :anvil do
    resources :backups, only: [ :index, :show ] do
      member do
        post :restore
        get :browse
      end
      collection do
        post :trigger
        get :chart_data
      end
    end
    resources :schedules, only: [ :index, :create, :destroy ] do
      member do
        patch :toggle
      end
    end
  end

  # Coming soon stubs
  get "/bellows" => "bellows#index"
  get "/flame" => "flame#index"
  get "/tongs" => "tongs#index"
  # Crucible — creative tools (forge melt bridge)
  get "/crucible" => "crucible#index"
  post "/crucible/chords" => "crucible#chords"
  post "/crucible/palette" => "crucible#palette"
  post "/crucible/diagram" => "crucible#diagram"
  get "/bridge" => "bridge#index"

  root "dashboard#show"
end
