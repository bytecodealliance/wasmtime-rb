# frozen_string_literal: true

require_relative "wasmtime/version"

module Wasmtime
end

# Tries to require the extension for the given Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "wasmtime/#{Regexp.last_match(1)}/wasmtime_rb"
rescue LoadError
  require "wasmtime/wasmtime_rb"
end
