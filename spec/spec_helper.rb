# frozen_string_literal: true

require "wasmtime"

module SpecHelpers
  def compile(wat)
    data = {}
    config = Wasmtime::Config.new
    engine = Wasmtime::Engine.new(config)
    store = Wasmtime::Store.new engine, data
    mod = Wasmtime::Module.new engine, wat
    Wasmtime::Instance.new(store, mod)
  end
end

RSpec.configure do |config|
  config.filter_run focus: true
  config.run_all_when_everything_filtered = true
  if ENV["CI"]
    config.before(focus: true) { raise "Do not commit focused tests (`fit` or `focus: true`)" }
  end

  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  if ENV["GC_STRESS"]
    config.around :each do |ex|
      GC.stress = true
      ex.run
    ensure
      GC.stress = false
    end
  end

  config.include SpecHelpers
end
