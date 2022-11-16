# frozen_string_literal: true

require_relative "wasmtime/version"

# Tries to require the extension for the given Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "wasmtime/#{Regexp.last_match(1)}/ext"
rescue LoadError
  require "wasmtime/ext"
end

module Wasmtime
  class Error < StandardError; end

  class ConversionError < Error; end
end
