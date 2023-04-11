require "spec_helper"

module Wasmtime
  RSpec.describe Func do
    describe ".new" do
      it "accepts block" do
        func = build_func([], []) {}
        func.call
      end

      it "raises without a block" do
        expect { build_func([], []) }
          .to raise_error(ArgumentError)
      end

      it "accepts supported Wasm types" do
        supported_types = [:i32, :i64, :f32, :f64, :v128, :funcref, :externref]
        supported_types.each do |type|
          func = build_func([type], []) {}
          expect(func.params).to eq([type])
          expect(func.results).to eq([])

          func = build_func([], [type]) {}
          expect(func.params).to eq([])
          expect(func.results).to eq([type])
        end
      end

      it "rejects unknown symbols" do
        expect { build_func([:nope], []) {} }
          .to raise_error(ArgumentError, /expected one of \[:i32, :i64, :f32, :f64, :v128, :funcref, :externref\], got :nope/)
      end

      it "rejects non-symbols" do
        expect { build_func(nil, nil) {} }.to raise_error(TypeError)
        expect { build_func([1], [2]) {} }.to raise_error(ArgumentError)
      end
    end

    describe ".call" do
      it "ignores the proc's return value when func has no results" do
        func = build_func([], []) { 1 }
        expect(func.call).to be_nil
      end

      it "accepts singlement element for single result" do
        func = build_func([], [:i32]) { 1 }
        expect(func.call).to eq(1)
      end

      it "accepts array of 1 element for single result" do
        func = build_func([], [:i32]) { [1] }
        expect(func.call).to eq(1)
      end

      it "rejects mismatching arguments size" do
        func = build_func([:i32, :i32], []) {}
        expect { func.call }.to raise_error(ArgumentError, /wrong number of arguments \(given 0, expected 2\)/)
      end

      it "rejects mismatching argument type" do
        func = build_func([:i32], []) {}
        expect { func.call("foo") }.to raise_error(TypeError, /\(param index 0\)/)
      end

      it "rejects mismatching results size" do
        func = build_func([], [:i32]) { [1, 2] }
        expect { func.call }.to raise_error(Wasmtime::ResultError, /wrong number of results \(given 2, expected 1\)/)
      end

      it "rejects unsafe result length vectors" do
        results = Array.new(Func::MAX_RESULTS + 1) { :i32 }

        expect do
          build_func([], results) { nil }
        end.to raise_error(ArgumentError, "too many results (max is 174, got 175)")
      end

      it "rejects mismatching result type" do
        func = build_func([], [:i32, :i32]) { [1, nil] }
        expect { func.call }.to raise_error(Wasmtime::ResultError) do |error|
          expect(error.message).to match(/no implicit conversion of nil into Integer/)
          expect(error.message).to match(/result at index 1/)
          expect(error.message).to match(/func_spec.rb:\d+/)
        end
      end

      it "re-enters into Wasm from Ruby" do
        called = false
        func1 = Func.new(store, [], []) { called = true }
        func2 = Func.new(store, [], []) { func1.call }
        func2.call
        expect(called).to be true
      end

      it "sends caller as first argument" do
        called = false
        store_data = BasicObject.new

        store = Store.new(engine, store_data)
        func = Func.new(store, [:i32], []) do |caller, _|
          called = true
          expect(caller).to be_instance_of(Caller)
          expect(caller.store_data).to equal(store_data)
        end

        func.call(1)
        expect(called).to be true
      end
    end

    describe "Caller" do
      it "exposes memory and func for the duration of the call only" do
        mod = Module.new(engine, <<~WAT)
          (module
            (import "" "" (func))
            (import "" "" (func))
            (memory (export "mem") 1)
            (export "f1_export" (func 1))
            (table (export "table") 1 funcref)
            (start 0))
        WAT
        store = Store.new(engine)
        calls = 0

        mem = nil
        f1_export = nil
        caller = nil

        f0 = Func.new(store, [], []) do |c|
          caller = c
          calls += 1

          mem = caller.export("mem").to_memory
          mem.write(0, "foo")
          expect(mem.read(0, 3)).to eq("foo")

          f1_export = caller.export("f1_export").to_func
          f1_export.call

          table_export = caller.export("table").to_table
          expect(table_export).to be_instance_of(Table)
        end
        f1 = Func.new(store, [], []) { calls += 1 }

        Instance.new(store, mod, [f0, f1])
        expect(calls).to eq(2)

        message = "Caller outlived its Func execution"
        expect { caller.export("f1_export") }.to raise_error(Wasmtime::Error, message)
        expect { mem.read(0, 3) }.to raise_error(Wasmtime::Error, message)
        expect { f1_export.call }.to raise_error(Wasmtime::Error, message)
      end
    end

    private

    def build_func(params, results, &block)
      store = Store.new(engine, Object.new)
      Func.new(store, params, results, &block)
    end
  end
end
