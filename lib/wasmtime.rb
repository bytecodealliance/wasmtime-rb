# frozen_string_literal: true

require_relative "wasmtime/version"

module Wasmtime
  # @note Support for Wasm components in the Ruby bindings is experimental. APIs may change in the future.
  module Component
    # Defining the `Component` module in Ruby ensures YARD pick it up.
  end
end

# Tries to require the extension for the given Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "wasmtime/#{Regexp.last_match(1)}/wasmtime_rb"
rescue LoadError
  require "wasmtime/wasmtime_rb"
end
