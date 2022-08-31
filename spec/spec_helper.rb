# frozen_string_literal: true

require "wasmtime"

RSpec.configure do |config|
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
end
