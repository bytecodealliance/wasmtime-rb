require "securerandom"

# Note: the heavy usage of `without_gc_stress` is to make the specs reasonably
# performant to run locally.

module Wasmtime
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

      n_times.times do |i|
        expect { func.call }.to raise_error(StandardError, /^hello\h{12}$/)
      end

      expect(call_times).to eq(n_times)
    end

    it "ensures values are never GC'd" do
      store = Store.new(engine, Object.new)
      big_array = without_gc_stress { Array.new(1024) { :i32 } }
      expected_result = without_gc_stress { Array.new(1024) { |i| i.to_s.to_i } }

      func = Func.new(store, [], big_array) { Array.new(1024) { |i| i } }

      n_times.times do
        expect(func.call).to eq(expected_result)
      end
    end
      end
    end

    def n_times
      ENV["GC_STRESS"] ? 2 : 1000
    end
  end
end
