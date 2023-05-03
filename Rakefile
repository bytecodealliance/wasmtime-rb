# frozen_string_literal: true

require "standard/rake"

GEMSPEC = Gem::Specification.load("wasmtime.gemspec")

task build: "pkg:ruby"

task default: %w[env:dev compile spec] + (ENV["CI"] ? %w[standard] : [])
