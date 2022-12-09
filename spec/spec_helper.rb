# frozen_string_literal: true

require "wasmtime"

Dir["./spec/support/**/*.rb"].sort.each { |f| require f }

RSpec.shared_context("default lets") do
  let(:engine_config) { Wasmtime::Config.new }
  let(:engine) { Wasmtime::Engine.new(engine_config) }
  let(:store_data) { {} }
  let(:store) { Wasmtime::Store.new(engine, store_data) }
  let(:wat) { "(module)" }

  def compile(wat)
    mod = Wasmtime::Module.new(engine, wat)
    Wasmtime::Instance.new(store, mod)
  end
end

RSpec.shared_context(:tmpdir) do
  let(:tmpdir) { Dir.mktmpdir }

  after(:each) do
    FileUtils.rm_rf(tmpdir)
  rescue Errno::EACCES => e
    warn "WARN: Failed to remove #{tmpdir} (#{e})"
  end
end

module WasmFixtures
  include Wasmtime
  extend self

  def wasi_debug
    @wasi_debug_module ||= Module.from_file(Engine.new(Wasmtime::Config.new), "spec/fixtures/wasi-debug.wasm")
  end
end

RSpec.configure do |config|
  config.include(ForkHelper)

  config.filter_run focus: true
  config.run_all_when_everything_filtered = true
  if ENV["CI"]
    config.before(focus: true) { raise "Do not commit focused tests (`fit` or `focus: true`)" }
  end

  config.include_context("default lets")

  # So memcheck steps can still pass if RSpec fails
  config.failure_exit_code = ENV.fetch("RSPEC_FAILURE_EXIT_CODE", 1).to_i
  config.default_formatter = ENV.fetch("RSPEC_FORMATTER") do
    config.files_to_run.one? ? "doc" : "progress"
  end

  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status" unless ENV["CI"]

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  if ENV["GC_STRESS"] == "1"
    config.around :each do |ex|
      GC.stress = true
      ex.run
    ensure
      GC.stress = false
    end
  end

  if ENV["VALGRIND"] == "1"
    config.around(:each) do |ex|
      ex.run
    rescue => e
      if e.message.include?("mmap failed to allocate")
        pending "Valgrind has a bug that causes mmap to fail"
      else
        raise
      end
    end
  end
end

at_exit { GC.start(full_mark: true) }
