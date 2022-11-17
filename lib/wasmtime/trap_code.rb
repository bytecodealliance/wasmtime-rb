module Wasmtime
  module TrapCode
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
    UNKNOWN = :unknown
  end
end
