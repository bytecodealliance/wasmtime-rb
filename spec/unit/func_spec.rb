require "spec_helper"

module Wasmtime
  RSpec.describe Func do
    it "calls the passed-in proc proc" do
      runs = 0
      func = build_func([], [], -> { runs += 1 })
      expect { func.call([]) }.to change { runs }.by(1)
      expect { func.call([]) }.to change { runs }.by(1)
    end

    it("converts i32 back and forth") { expect(roundtrip_value(:i32, 4)).to eq(4) }
    it("converts i64 back and forth") { expect(roundtrip_value(:i64, 2**40)).to eq(2**40) }
    it("converts f32 back and forth") { expect(roundtrip_value(:f32, 5.5)).to eq(5.5) }
    it("converts f64 back and forth") { expect(roundtrip_value(:f64, 5.5)).to eq(5.5) }
    it("converts nil externref back and forth") { expect(roundtrip_value(:externref, nil)).to be_nil }
    it("converts string externref back and forth") { expect(roundtrip_value(:externref, "foo")).to eq("foo") }

    it "converts BasicObject externref back and forth" do
      obj = BasicObject
      expect(roundtrip_value(:externref, obj)).to equal(obj)
    end

    it "converts ref.null into nil" do
      instance = compile(<<~WAT)
        (module
          (func (export "main") (result externref)
            ref.null extern))
      WAT
      expect(instance.invoke("main", [])).to be_nil
    end

    it "ignores the proc's return value when func has no results" do
      func = build_func([], [], -> { 1 })
      expect(func.call([])).to be_nil
    end

    it "accepts array of 1 element for single result" do
      func = build_func([], [:i32], -> { [1] })
      expect(func.call([])).to eq(1)
    end

    it "rejects mismatching results size" do
      func = build_func([], [:i32], -> { [1, 2] })
      expect { func.call([]) }.to raise_error(Wasmtime::Error, /wrong number of results \(given 2, expected 1\)/)
    end

    it "rejects mismatching result type" do
      func = build_func([], [:i32], -> { [nil] })
      expect { func.call([]) }.to raise_error(Wasmtime::Error)
    end

    it "tells you which result failed to convert in the error message" do
      skip("TODO!")
    end

    it "rejects mismatching params size" do
      func = build_func([:i32], [], ->(_, _) {})
      expect { func.call([1]) }.to raise_error(ArgumentError, /wrong number of arguments \(given 1, expected 2\)/)
    end

    it "bubbles the exception on with call" do
      error_class = Class.new(StandardError)
      func = build_func([], [], -> { raise error_class })
      expect { func.call([]) }.to raise_error(error_class)
      # Run a second time to catch already borrowed issues
      expect { func.call([]) }.to raise_error(error_class)
    end

    it "bubbles the exception on start" do
      error_class = Class.new(StandardError)
      func = Func.new(store, FuncType.new([], []), true, -> { raise error_class })
      mod = Wasmtime::Module.new(engine, <<~WAT)
        (module
          (import "" "" (func))
          (start 0))
      WAT

      expect { Wasmtime::Instance.new(store, mod, [func]) }
        .to raise_error(error_class)
    end

    private

    def roundtrip_value(type, value)
      build_func([type], [type], ->(arg) { arg })
        .call([value])
    end

    def build_func(params, results, impl)
      store = Store.new(engine, {})
      Func.new(store, FuncType.new(params, results), false, impl)
    end
  end
end
