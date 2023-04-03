require "securerandom"

module Wasmtime
  # Note: the heavy usage of `without_gc_stress` is to make the specs reasonably
  # performant to run locally. These specs are meant to to smoke test the gem by
  # exercising the most insane edge cases we can think of.
  RSpec.describe "Crash" do
    # see https://github.com/bytecodealliance/wasmtime-rb/issues/156
    it "ensures result errors are never GC'd" do
      store = Store.new(engine, Object.new)
      func = Func.new(store, [], [:i32, :i32]) { [1, nil] }

      n_times.times do
        func.call
      rescue Wasmtime::ResultError
      end
    end

    it "ensures user exceptions are never GC'd" do
      store = Store.new(engine, Object.new)
      call_times = 0
      func = Func.new(store, [], [:i32, :i32]) do
        call_times += 1
        # most GC-able exception ever?
        raise Class.new(StandardError).new((+"hello") + SecureRandom.hex(6))
      end

    #   n_times.times do |i|
    #     expect { func.call }.to raise_error(StandardError, /^hello\h{12}$/)
    #   end

    #   expect(call_times).to eq(n_times)
    # end

    # it "ensures values are never GC'd" do
    #   n_times = n_times(max: 100)
    #   store = Store.new(engine, Object.new)
    #   big_array = without_gc_stress { Array.new(256) { :i32 } }
    #   expected_result = without_gc_stress { Array.new(256) { |i| i.to_s.to_i } }

    #   func = Func.new(store, [], big_array) { Array.new(256) { |i| i } }

    #   n_times.times do
    #     expect(func.call).to eq(expected_result)
    #   end
    # end

    # it "ensures params are never GC'd" do
    #   n_times = n_times(max: 100)
    #   arrayish = Struct.new(:to_ary)
    #   size = 8
    #   params_type = without_gc_stress { Array.new(size) { :i32 } }
    #   results_type = without_gc_stress { arrayish.new(Array.new(size) { :i32 }) }
    #   params = build_sequential_int_array(size)
    #   called_times = 0

    #   store = Store.new(engine, Object.new)
    #   func = Func.new(store, params_type, results_type) do |_, *args|
    #     called_times += 1

    #     fiber = Fiber.new do
    #       result = without_gc_stress { Struct.new(:to_ary).new([*args]) }
    #       Fiber.yield "yielded"
    #       Fiber.yield result
    #     end

    #     result = fiber.resume
    #     without_gc_stress { expect(result).to eq("yielded") }
    #     fiber.resume
    #   end

    #   expected_result = build_sequential_int_array(size)

    #   n_times.times do
    #     result = func.call(*params)

    #     without_gc_stress do
    #       expect(result).to eq(expected_result)
    #     end
    #   end

    #   expect(called_times).to eq(n_times)
    # end

    def n_times(max: 1000)
      ENV["GC_STRESS"] ? 1 : max
    end

    def build_sequential_int_array(size)
      without_gc_stress { Array.new(size) { |i| i } }
    end
  end
end
