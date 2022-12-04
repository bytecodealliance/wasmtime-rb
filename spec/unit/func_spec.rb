require "spec_helper"

module Wasmtime
  RSpec.describe Func do
    describe ".new" do
      it "accepts block" do
        store = Store.new(engine, {})
        func = Func.new(store, FuncType.new([], [])) {}
        func.call
      end

      it "raises without a block" do
        expect { build_func([], []) }
          .to raise_error(ArgumentError)
      end
    end

    describe ".call" do
      it "ignores the proc's return value when func has no results" do
        func = build_func([], []) { 1 }
        expect(func.call).to be_nil
      end

      it "accepts array of 1 element for single result" do
        func = build_func([], [:i32]) { [1] }
        expect(func.call).to eq(1)
      end

      it "rejects mismatching results size" do
        func = build_func([], [:i32]) { [1, 2] }
        expect { func.call }.to raise_error(Wasmtime::Error, /wrong number of results \(given 2, expected 1\)/)
      end

      it "rejects mismatching result type" do
        func = build_func([], [:i32]) { [nil] }
        expect { func.call }.to raise_error(Wasmtime::Error)
      end

      it "tells you which result failed to convert in the error message" do
        skip("TODO!")
      end

      it "re-enters into Wasm from Ruby" do
        called = false
        func1 = Func.new(store, FuncType.new([], [])) { called = true }
        func2 = Func.new(store, FuncType.new([], [])) { func1.call }
        func2.call
        expect(called).to be true
      end

      it "sends caller as first argument" do
        called = false
        store_data = BasicObject.new

        store = Store.new(engine, store_data)
        func = Func.new(store, FuncType.new([:i32], [])) do |caller, _|
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
        engine = Engine.new
        mod = Module.new(engine, <<~WAT)
          (module
            (import "" "" (func))
            (import "" "" (func))
            (memory (export "mem") 1)
            (export "f1_export" (func 1))
            (start 0))
        WAT
        store = Store.new(engine)
        calls = 0

        mem = nil
        f1_export = nil
        caller = nil

        f0 = Func.new(store, FuncType.new([], [])) do |c|
          caller = c
          calls += 1

          mem = caller.export("mem").to_memory
          mem.write(0, "foo")
          expect(mem.read(0, 3)).to eq("foo")

          f1_export = caller.export("f1_export").to_func
          f1_export.call
        end
        f1 = Func.new(store, FuncType.new([], [])) { calls += 1 }

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
      store = Store.new(engine, {})
      Func.new(store, FuncType.new(params, results), &block)
    end
  end
end
