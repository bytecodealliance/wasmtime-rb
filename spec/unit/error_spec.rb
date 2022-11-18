require "spec_helper"

module Wasmtime
  RSpec.describe "Error" do
    let(:error_class) { Class.new(StandardError) }

    context "on instance start" do
      context "from Instance.new" do
        it "bubbles host exception" do
          func = Func.new(store, FuncType.new([], [])) { raise error_class }

          expect { Wasmtime::Instance.new(store, module_import_func_start, [func]) }
            .to raise_error(error_class)
        end

        it "bubbles trap" do
          expect { Wasmtime::Instance.new(store, module_trapping_on_start, []) }
            .to raise_error(Trap)
        end
      end

      context "from Linker#instantiate" do
        it "bubbles host exception" do
          linker = Linker.new(engine)
          linker.func_new("", "", FuncType.new([], [])) { raise error_class }
          store = Store.new(engine)

          expect { linker.instantiate(store, module_import_func_start) }.to raise_error(error_class)
        end

        it "bubbles trap" do
          linker = Linker.new(engine)
          store = Store.new(engine)

          expect { linker.instantiate(store, module_trapping_on_start) }
            .to raise_error(Trap)
        end
      end
    end

    context "on call" do
      context "from Func#call" do
        it "bubbles host exception" do
          store = Store.new(engine)
          func = Func.new(store, FuncType.new([], [])) { raise error_class }

          expect { func.call }.to raise_error(error_class)
        end

        it "bubbles trap" do
          func = Instance.new(Store.new(engine), module_trapping_on_func)
            .export("f")
            .to_func

          expect { func.call }.to raise_error(Trap)
        end
      end

      context "from Instance#invoke" do
        it "bubbles host exception" do
          store = Store.new(engine)
          mod = Module.new(engine, <<~WAT)
            (module
              (import "" "" (func))
              (export "f" (func 0)))
          WAT
          func = Func.new(store, FuncType.new([], [])) { raise error_class }
          instance = Wasmtime::Instance.new(store, mod, [func])

          expect { instance.invoke("f") }.to raise_error(error_class)
        end

        it "bubbles trap" do
          instance = Instance.new(Store.new(engine), module_trapping_on_func)
          expect { instance.invoke("f") }.to raise_error(Trap)
        end
      end
    end

    def module_import_func_start
      Wasmtime::Module.new(engine, <<~WAT)
        (module
          (import "" "" (func))
          (start 0))
      WAT
    end

    def module_trapping_on_start
      Wasmtime::Module.new(engine, <<~WAT)
        (module
          (func unreachable)
          (start 0))
      WAT
    end

    def module_trapping_on_func
      Wasmtime::Module.new(engine, <<~WAT)
        (module
          (func (export "f") unreachable))
      WAT
    end
  end
end
