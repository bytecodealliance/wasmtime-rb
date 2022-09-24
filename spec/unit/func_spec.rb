require "spec_helper"

module Wasmtime
  RSpec.describe Func do
    it "calls the passed-in proc proc" do
      runs = 0
      instance = instance_for_func([], [], -> { runs += 1 })
      GC.start

      instance.invoke("f", [])
      expect(runs).to eq(1)

      instance.invoke("f", [])
      expect(runs).to eq(2)
    end

    it("converts i32 back and forth") { expect(roundtrip_value(:i32, 4)).to eq(4) }
    it("converts i64 back and forth") { expect(roundtrip_value(:i64, 2**40)).to eq(2**40) }
    it("converts f32 back and forth") { expect(roundtrip_value(:f32, 5.5)).to eq(5.5) }
    it("converts f64 back and forth") { expect(roundtrip_value(:f64, 5.5)).to eq(5.5) }

    it "ignores the proc's return value when func has no results" do
      instance = instance_for_func([], [], -> { 1 })
      expect(instance.invoke("f", [])).to be_nil
    end

    it "accepts array of 1 element for single result" do
      instance = instance_for_func([], [:i32], -> { [1] })
      expect(instance.invoke("f", [])).to eq(1)
    end

    it "rejects mismatching results size" do
      instance = instance_for_func([], [:i32], -> { [1, 2] })
      expect { instance.invoke("f", []) }.to raise_error(Wasmtime::Error, /wrong number of results \(given 2, expected 1\)/)
    end

    it "rejects mismatching result type" do
      instance = instance_for_func([], [:i32], -> { [nil] })
      expect { instance.invoke("f", []) }.to raise_error(Wasmtime::Error)
    end

    it "tells you which result failed to convert in the error message" do
      skip("TODO!")
    end

    it "rejects mismatching params size" do
      instance = instance_for_func([:i32], [], ->(_, _) {})
      expect { instance.invoke("f", [1]) }.to raise_error(Wasmtime::Error, /wrong number of arguments \(given 1, expected 2\)/)
    end

    it "bubbles the exception on with invoke" do
      skip("TODO!")
      expect { instance_for_func([], [], -> { raise "Ooops!" }).invoke("f", []) }
        .to raise_error(StandardError, "Ooops!")
    end

    it "bubbles the exception on start" do
      skip("TODO!")
      func = Func.new(store, FuncType.new([], []), true, -> { raise "Ooops!" })
      mod = Wasmtime::Module.new(engine, <<~WAT)
        (module
          (import "" "" (func))
          (start 0))
      WAT

      expect { Wasmtime::Instance.new(store, mod, [func]) }
        .to raise_error(StandardError, "Ooops!")
    end

    private

    def roundtrip_value(type, value)
      instance_for_func([type], [type], ->(arg) { arg })
        .invoke("f", [value])
    end

    def instance_for_func(params, results, impl)
      func = Func.new(store, FuncType.new(params, results), false, impl)
      mod = Wasmtime::Module.new(engine, <<~WAT)
        (module
          (import "" "" (func (param #{params.join(" ")}) (result #{results.join(" ")})))
          (export "f" (func 0)))
      WAT
      Wasmtime::Instance.new(store, mod, [func])
    end
  end
end
