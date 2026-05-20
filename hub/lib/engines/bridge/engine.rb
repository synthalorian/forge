module Bridge
  class Engine < ::Rails::Engine
    isolate_namespace Bridge
  end
end