# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"

RSpec::Core::RakeTask.new(:spec)

require "standard/rake"

GEMSPEC = Gem::Specification.load("wasmtime.gemspec")

task build: "pkg:ruby"

task default: %i[compile spec standard]
