# frozen_string_literal: true

require "standard/rake"

GEMSPEC = Gem::Specification.load("wasmtime.gemspec")

task build: "pkg:ruby"

task default: %i[compile spec standard]
