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
        expect { func.call("foo") }.to raise_error(TypeError, /\(param at index 0\)/)
      end

      it "rejects mismatching results size" do
        func = build_func([], [:i32]) { [1, 2] }
        expect { func.call }.to raise_error(Wasmtime::ResultError, /wrong number of results \(given 2, expected 1\)/)
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

      it "disallows cross-store funcref arg" do
        store2 = Store.new(engine, {})
        func = Func.new(store, [:funcref], []) {}
        store2_func = Func.new(store2, [], []) {}

        expect { func.call(store2_func) }.to raise_error(Wasmtime::Error, /argument type mismatch/)
      end

      it "disallows cross-store funcref result" do
        store2 = Store.new(engine, {})
        store2_func = Func.new(store2, [], []) {}
        func = Func.new(store, [], [:funcref]) { |_, funcref| store2_func }

        expect { func.call }.to raise_error(Wasmtime::Error, /function attempted to return an incompatible value/)
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

    describe "#call with gvl: false" do
      it "still returns correct results" do
        store = Store.new(engine)
        mod = Module.new(engine, <<~WAT)
          (module (func (export "add") (param i32 i32) (result i32)
            (i32.add (local.get 0) (local.get 1))))
        WAT
        func = Instance.new(store, mod).export("add").to_func(gvl: false)
        expect(func.call(2, 3)).to eq(5)
      end

      it "releases the GVL so other Ruby threads run during the call" do
        mod = Module.new(engine, <<~WAT)
          (module
            (import "env" "mark" (func $mark))
            (func (export "spin") (param $n i64) (result i64)
              (local $i i64)
              (call $mark)
              (block $done
                (loop $loop
                  (br_if $done (i64.ge_u (local.get $i) (local.get $n)))
                  (local.set $i (i64.add (local.get $i) (i64.const 1)))
                  (br $loop)))
              (call $mark)
              (local.get $i)))
        WAT

        counter = 0
        running = true
        sibling = Thread.new { counter += 1 while running }

        marks = []
        store = Store.new(engine)
        mark = Func.new(store, [], []) { marks << counter }
        linker = Linker.new(engine)
        linker.define(store, "env", "mark", mark)
        instance = linker.instantiate(store, mod)

        instance.export("spin").to_func(gvl: false).call(500_000_000)

        running = false
        sibling.join

        expect(marks.length).to eq(2)
        expect(marks.last).to be > marks.first
      end

      it "re-acquires the GVL for Ruby host callbacks during a released call" do
        expect(run_released_with_callback(host_callback_module, spin: 1_000)).to eq(42)
      end

      it "stays correct with host callbacks across threads under GC stress" do
        mod = host_callback_module

        results = with_gc_stress do
          10.times.map do
            Thread.new { run_released_with_callback(mod, spin: 50_000) }
          end.map(&:value)
        end

        expect(results).to eq(Array.new(10, 42))
      end

      it "bubbles a host-callback exception raised during a released call" do
        error_class = Class.new(StandardError)
        store = Store.new(engine)
        raising = Func.new(store, [:i32], [:i32]) { raise error_class, "boom" }
        linker = Linker.new(engine)
        linker.define(store, "env", "host_add", raising)
        func = linker.instantiate(store, host_callback_module).export("run").to_func(gvl: false)

        expect { func.call(1_000) }.to raise_error(error_class, "boom")
      end

      it "raises ResultError for a wrong-arity callback result during a released call" do
        store = Store.new(engine)
        bad = Func.new(store, [:i32], [:i32]) { |_caller, _x| [1, 2] }
        linker = Linker.new(engine)
        linker.define(store, "env", "host_add", bad)
        func = linker.instantiate(store, host_callback_module).export("run").to_func(gvl: false)

        expect { func.call(1_000) }.to raise_error(Wasmtime::ResultError, /wrong number of results/)
      end

      it "raises Trap for a Wasm trap during a released call" do
        store = Store.new(engine)
        mod = Module.new(engine, <<~WAT)
          (module (func (export "boom") unreachable))
        WAT
        func = Instance.new(store, mod).export("boom").to_func(gvl: false)

        expect { func.call }.to raise_error(Trap)
      end

      it "bubbles host-callback exceptions across threads under GC stress" do
        error_class = Class.new(StandardError)

        results = with_gc_stress do
          10.times.map do
            Thread.new do
              store = Store.new(engine)
              raising = Func.new(store, [:i32], [:i32]) { raise error_class, "boom" }
              linker = Linker.new(engine)
              linker.define(store, "env", "host_add", raising)
              func = linker.instantiate(store, host_callback_module).export("run").to_func(gvl: false)
              begin
                func.call(10_000)
                :no_error
              rescue error_class
                :raised
              end
            end.value
          end
        end

        expect(results).to eq(Array.new(10, :raised))
      end
    end

    private

    def build_func(params, results, &block)
      store = Store.new(engine, Object.new)
      Func.new(store, params, results, &block)
    end

    def host_callback_module
      Module.new(engine, <<~WAT)
        (module
          (import "env" "host_add" (func $host_add (param i32) (result i32)))
          (func (export "run") (param $n i64) (result i32)
            (local $i i64)
            (block $done
              (loop $loop
                (br_if $done (i64.ge_u (local.get $i) (local.get $n)))
                (local.set $i (i64.add (local.get $i) (i64.const 1)))
                (br $loop)))
            (call $host_add (i32.const 41))))
      WAT
    end

    def run_released_with_callback(mod, spin:)
      store = Store.new(engine)
      host_add = Func.new(store, [:i32], [:i32]) { |_caller, x| x.to_s.to_i + 1 }
      linker = Linker.new(engine)
      linker.define(store, "env", "host_add", host_add)
      instance = linker.instantiate(store, mod)
      instance.export("run").to_func(gvl: false).call(spin)
    end
  end
end
