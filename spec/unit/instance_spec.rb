require "spec_helper"

module Wasmtime
  RSpec.describe Instance do
    describe "#new" do
      it "raises a TypeError when receiving invalid imports" do
        mod = Wasmtime::Module.new(engine, "(module)")

        expect { Wasmtime::Instance.new(store, mod, "not an array") }
          .to raise_error(TypeError, %(unexpected extern: "not an array"))
      end

      it "accepts nil for imports" do
        mod = Wasmtime::Module.new(engine, "(module)")

        expect { Wasmtime::Instance.new(store, mod, nil) }
          .not_to raise_error
      end
    end

    it "exposes the exports" do
      instance = compile <<~WAT
        (module
          (memory $module/mem 1)
          (func $module/hello (result i32)
            i32.const 1
          )
          (export "hello" (func $module/hello))
          (export "mem" (memory $module/mem))
        )
      WAT

      exports = instance.exports
      type_names = exports.transform_values(&:type_name)

      expect(exports).to include(hello: be_a(Export), mem: be_a(Export))
      expect(type_names).to eq(hello: :func, mem: :memory)
    end

    describe "invoke" do
      it "returns nil when func has no return value" do
        instance = compile(<<~WAT)
          (module
            (func (export "main")))
        WAT
        expect(instance.invoke("main", [])).to be_nil
      end

      it "returns a value when func has single return value" do
        instance = compile(<<~WAT)
          (module
            (func (export "main") (result i32)
              i32.const 42))
        WAT
        expect(instance.invoke("main", [])).to eq(42)
      end

      it "returns an array when func has multiple return values" do
        instance = compile(<<~WAT)
          (module
            (func (export "main") (result i32) (result i32)
              i32.const 42
              i32.const 43))
        WAT
        expect(instance.invoke("main", [])).to eq([42, 43])
      end

      it "calls a func with i32" do
        expect(invoke_identity_function("i32", 1)).to eq(1)
      end

      it "calls a func with i32 overflow" do
        expect { invoke_identity_function("i32", 2**50) }.to raise_error(RangeError)
      end

      it "calls a func with i64" do
        expect(invoke_identity_function("i64", 2**50)).to eq(2**50)
      end

      it "calls a func with i64 overflow" do
        expect { invoke_identity_function("i64", 2**65) }.to raise_error(RangeError)
      end

      it "calls a func with f32" do
        expect(invoke_identity_function("f32", 2.0)).to eq(2.0)
      end

      it "calls a func with f32 overflow" do
        expect(invoke_identity_function("f32", 5 * 10**40)).to eq(Float::INFINITY)
      end

      it "calls a func with f64" do
        expect(invoke_identity_function("f64", 2.0)).to eq(2.0)
      end

      it "calls a func with f32 overflow" do
        expect(invoke_identity_function("f32", 5 * 10**40)).to eq(Float::INFINITY)
      end
    end

    it "imports memory" do
      mod = Module.new(engine, <<~WAT)
        (module
          (import "" "" (memory 1)))
      WAT
      memory = Memory.new(store, MemoryType.new(1))
      Wasmtime::Instance.new(store, mod, [memory])
    end

    private

    def invoke_identity_function(type, arg)
      instance = compile(<<~WAT)
        (module
          (func (export "main") (param #{type}) (result #{type})
            local.get 0))
      WAT
      instance.invoke("main", [arg])
    end
  end
end
