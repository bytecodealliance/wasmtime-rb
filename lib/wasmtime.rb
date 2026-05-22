# frozen_string_literal: true

require_relative "wasmtime/version"

module Wasmtime
end

# Tries to require the extension for the given Ruby version first.
# Only fall back to the non-versioned extension when the versioned extension is
# absent. If the versioned extension exists but fails to load, re-raise the
# original error so native loader failures are not hidden.
ruby_version = RUBY_VERSION[/\A\d+\.\d+/]
extension = "wasmtime/#{ruby_version}/wasmtime_rb"

begin
  require extension
rescue LoadError => e
  raise unless e.path == extension

  require "wasmtime/wasmtime_rb"
end
