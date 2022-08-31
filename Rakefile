# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"

RSpec::Core::RakeTask.new(:spec)

require "standard/rake"

require "rake/extensiontask"

task build: :compile

Rake::ExtensionTask.new("ext") do |ext|
  ext.lib_dir = "lib/wasmtime"
  ext.ext_dir = "ext"
end

task default: %i[compile spec standard]
