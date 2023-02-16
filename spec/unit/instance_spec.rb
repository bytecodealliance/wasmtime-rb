require "spec_helper"

module Wasmtime
  RSpec.describe Instance do
    describe "#new" do
      it "raises a TypeError when receiving invalid imports" do
        mod = Wasmtime::Module.new(engine, "(module)")

        expect { Wasmtime::Instance.new(store, mod, [:not_extern]) }
          .to raise_error(TypeError, "unexpected extern: :not_extern")
      end

      it "accepts nil for imports" do
        mod = Wasmtime::Module.new(engine, "(module)")

        expect { Wasmtime::Instance.new(store, mod, nil) }
          .not_to raise_error
      end

      it "imports memory" do
        mod = Module.new(engine, <<~WAT)
          (module
            (import "" "" (memory 1)))
        WAT
        memory = Memory.new(store, min_size: 1)
        Wasmtime::Instance.new(store, mod, [memory])
      end
    end

    describe "#exports" do
      it "returns a Hash of Extern" do
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

        expect(instance.exports).to include("hello" => be_a(Extern), "mem" => be_a(Extern))
        expect(instance.exports["hello"].to_func).to be_a(Func)
        expect(instance.exports["mem"].to_memory).to be_a(Memory)
      end
    end

    describe "export" do
      it "returns a single Extern" do
        instance = compile(<<~WAT)
          (module
            (func (export "f")))
        WAT
        expect(instance.export("f").to_func).to be_a(Func)
      end
    end

    describe "invoke" do
      it "returns nil when func has no return value" do
        instance = compile(<<~WAT)
          (module
            (func (export "main")))
        WAT
        expect(instance.invoke("main")).to be_nil
      end

      it "returns a value when func has single return value" do
        instance = compile(<<~WAT)
          (module
            (func (export "main") (result i32)
              i32.const 42))
        WAT
        expect(instance.invoke("main")).to eq(42)
      end

      it "returns an array when func has multiple return values" do
        instance = compile(<<~WAT)
          (module
            (func (export "main") (result i32) (result i32)
              i32.const 42
              i32.const 43))
        WAT
        expect(instance.invoke("main")).to eq([42, 43])
      end
    end

    private

    def invoke_identity_function(type, arg)
      instance = compile(<<~WAT)
        (module
          (func (export "main") (param #{type}) (result #{type})
            local.get 0))
      WAT
      instance.invoke("main", arg)
    end
  end
end
