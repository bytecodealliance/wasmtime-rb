# frozen_string_literal: true

module Wasmtime
  class Error < StandardError; end

  # Raised when failing to convert the return value of a Ruby-backed Func to
  # Wasm types.
  class ResultError < Error; end

  # Raised when converting an {Wasmtime::Extern} to its concrete type fails.
  class ConversionError < Error; end

  # Raised on Wasm trap.
  class Trap < Error
    STACK_OVERFLOW = :stack_overflow
    MEMORY_OUT_OF_BOUNDS = :memory_out_of_bounds
    HEAP_MISALIGNED = :heap_misaligned
    TABLE_OUT_OF_BOUNDS = :table_out_of_bounds
    INDIRECT_CALL_TO_NULL = :indirect_call_to_null
    BAD_SIGNATURE = :bad_signature
    INTEGER_OVERFLOW = :integer_overflow
    INTEGER_DIVISION_BY_ZERO = :integer_division_by_zero
    BAD_CONVERSION_TO_INTEGER = :bad_conversion_to_integer
    UNREACHABLE_CODE_REACHED = :unreachable_code_reached
    INTERRUPT = :interrupt
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
