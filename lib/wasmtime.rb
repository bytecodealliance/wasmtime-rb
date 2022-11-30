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

  class Trap < Error
    STACK_OVERFLOW = :stack_overflow
    HEAP_MISALIGNED = :heap_misaligned
    TABLE_OUT_OF_BOUNDS = :table_out_of_bounds
    INDIRECT_CALL_TO_NULL = :indirect_call_to_null
    BAD_SIGNATURE = :bad_signature
    INTEGER_OVERFLOW = :integer_overflow
    INTEGER_DIVISION_BY_ZERO = :integer_division_by_zero
    BAD_CONVERSION_TO_INTEGER = :bad_conversion_to_integer
    UNREACHABLE_CODE_REACHED = :unreachable_code_reached
    INTERRUPT = :interrupt
    ALWAYS_TRAP_ADAPTER = :always_trap_adapter
    OUT_OF_FUEL = :out_of_fuel
    UNKNOWN = :unknown
  end

  # Raised when a WASI program terminates early by calling +exit+.
  class WasiExit < Error
    # @return [Integer] The system exit code.
    attr_reader(:code)

    def initialize(code)
      @code = code
    end

    # @return [String]
    def message
      "WASI exit with code #{code}"
    end
  end
end
