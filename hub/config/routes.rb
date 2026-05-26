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

  # Bellows — agent orchestration
  get "/bellows" => "bellows#index"
  get "/bellows/sessions" => "bellows#sessions"
  get "/bellows/sessions/:id" => "bellows#session_detail"
  delete "/bellows/sessions/:id" => "bellows#delete_session"
  post "/bellows/pipeline" => "bellows#run_pipeline"
  post "/bellows/strike" => "bellows#strike"
  get "/flame" => "flame#index"
  get "/flame/journal" => "flame#journal"
  get "/flame/journal/:id" => "flame#journal_entry"
  post "/flame/journal/search" => "flame#journal_search"
  post "/flame/search" => "flame#search_scripture"
  post "/flame/reference" => "flame#lookup_reference"
  get "/tongs" => "tongs#index"
  get "/tongs/diagnose" => "tongs#diagnose"
  get "/tongs/dotfiles" => "tongs#dotfiles"
  post "/tongs/dotfiles/track" => "tongs#track_dotfile"
  post "/tongs/dotfiles/restore" => "tongs#restore_dotfile"
  get "/tongs/services" => "tongs#services"
  # Crucible — creative tools (forge melt bridge)
  get "/crucible" => "crucible#index"
  post "/crucible/chords" => "crucible#chords"
  post "/crucible/palette" => "crucible#palette"
  post "/crucible/palette/upload" => "crucible#upload_palette_image"
  post "/crucible/diagram" => "crucible#diagram"
  get "/bridge" => "bridge#index"
  get "/bridge/sync" => "bridge#sync"
  post "/bridge/notify" => "bridge#send_notification"

  # Global search
  get "/search" => "search#show"

  root "dashboard#show"
end
