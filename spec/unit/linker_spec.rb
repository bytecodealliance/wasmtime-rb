require "spec_helper"

module Wasmtime
  RSpec.describe Linker do
    it "allow_shadowing" do
      linker = new_linker
      functype = FuncType.new([], [])
      linker.func_new("foo", "bar", functype, -> {})

      linker.allow_shadowing = false
      expect { linker.func_new("foo", "bar", functype, -> {}) }
        .to raise_error(Wasmtime::Error)

      linker.allow_shadowing = true
      expect { linker.func_new("foo", "bar", functype, -> {}) }
        .not_to raise_error
    end

    it "allow_unknown_exports" do
      mod = Module.new(engine, <<~WAT)
        (module
          (func (export "_start"))
          (memory (export "___") 1))
      WAT
      linker = new_linker
      linker.allow_unknown_exports = false
      expect { linker.module(store, "m", mod) }.to raise_error(Wasmtime::Error, /is not a function/)

      linker = new_linker
      linker.allow_unknown_exports = true
      expect(linker.module(store, "m", mod)).to be_nil
    end

    it "define_unknown_imports_as_traps" do
      mod = Module.new(engine, '(module (import "" "" (func)))')
      linker = new_linker
      expect { linker.instantiate(store, mod) }.to raise_error(Wasmtime::Error, /unknown import/)

      linker.define_unknown_imports_as_traps(mod)
      expect { linker.instantiate(store, mod) }.not_to raise_error
    end

    it "define memory" do
      linker = new_linker
      store = Store.new(engine)
      memory = Memory.new(store, MemoryType.new(1))
      linker.define("mod", "mem", memory)
      expect(linker.get(store, "mod", "mem")).to be_instance_of(Memory)
    end

    it "define func" do
      linker = new_linker
      store = Store.new(engine)
      func = Func.new(store, FuncType.new([], []), -> {})
      linker.define("mod", "fn", func)
      expect(linker.get(store, "mod", "fn")).to be_instance_of(Func)
    end

    it "func_new accepts proc or block" do
      functype = FuncType.new([], [])

      expect { new_linker.func_new("foo", "bar", functype, -> {}) {} }
        .to raise_error(ArgumentError, "provide block or proc argument, not both")

      expect { new_linker.func_new("foo", "bar", functype) }
        .to raise_error(ArgumentError, "provide block or proc argument")

      expect { new_linker.func_new("foo", "bar", functype, -> {}) }
        .not_to raise_error

      expect { new_linker.func_new("foo", "bar", functype) {} }
        .not_to raise_error
    end

    it "func_new imports can be called" do
      functype = FuncType.new([], [])
      calls = 0
      linker = new_linker
      linker.func_new("", "", functype, -> { calls += 1 })
      func = linker.get(Store.new(engine), "", "")
      expect { func.call([]) }.to change { calls }.by(1)
    end

    it "func_new sends caller when requested" do
      functype = FuncType.new([], [])
      calls = 0
      linker = new_linker
      linker.func_new("", "", functype, caller: true) do |caller|
        calls += 1
        expect(caller).to be_instance_of(Caller)
      end
      instance = linker.instantiate(Store.new(engine), func_reexport_module)

      expect { instance.invoke("f", []) }.to change { calls }.by(1)
    end

    it "get returns nil for undefined items" do
      linker = new_linker
      store = Store.new(engine)
      expect(linker.get(store, "nope", "nope")).to be_nil
    end

    it "get can return Func" do
      linker = new_linker
      linker.func_new("mod", "fn", FuncType.new([], [:i32]), -> { 42 })
      func = linker.get(Store.new(engine), "mod", "fn")
      expect(func).to be_instance_of(Func)
      expect(func.call([])).to eq(42)
    end

    it "module" do
      linker = new_linker
      store = Store.new(engine)
      mod1 = Module.new(engine, '(module (func (export "run") ))')
      linker.module(store, "instance1", mod1)

      mod2 = Module.new(engine, <<~WAT)
        (module
            (import "instance1" "run" (func $instance1_run))
            (func (export "run")))
      WAT

      instance = linker.instantiate(store, mod2)
      expect(instance).to be_instance_of(Instance)
      expect(instance.exports).to have_key(:run)
    end

    it "instance" do
      linker = new_linker
      mod = Module.new(engine, '(module (func (export "fn")))')
      linker.instance(store, "mod", Wasmtime::Instance.new(store, mod))
      expect(linker.get(store, "mod", "fn")).to be_truthy
    end

    it "module" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      expect(linker.get(store, "mod1", "fn1")).to be_truthy
    end

    it "alias" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      linker.alias("mod1", "fn1", "mod2", "fn2")
      expect(linker.get(store, "mod2", "fn2")).to be_truthy
    end

    it "alias_module" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      linker.alias_module("mod1", "mod2")
      expect(linker.get(store, "mod2", "fn1")).to be_truthy
    end

    it "instantiate" do
      linker = new_linker
      linker.func_new("", "", FuncType.new([], []), -> {})
      instance = linker.instantiate(Store.new(engine), func_reexport_module)
      expect(instance).to be_instance_of(Instance)
    end

    it "instantiate bubbles exceptions from the start func" do
      error_class = Class.new(StandardError)
      mod = Wasmtime::Module.new(engine, <<~WAT)
        (module
          (import "" "" (func))
          (start 0))
      WAT
      functype = FuncType.new([], [])
      linker = new_linker
      linker.func_new("", "", functype, -> { raise error_class })

      expect { linker.instantiate(Store.new(engine), mod) }
        .to raise_error(error_class)
    end

    it "get_default" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "")))'))
      linker.module(store, "mod2", Module.new(engine, '(module (memory (export "") 1))'))

      expect(linker.get_default(store, "mod1")).to be_instance_of(Func)
      expect { linker.get_default(store, "mod2") }.to raise_error(Wasmtime::Error, /not a function/)
    end

    it "instance and func gc stress" do
      calls = 0
      functype = FuncType.new([], [])
      linker = new_linker
      linker.func_new("", "", functype, -> { calls += 1 })
      instance = linker.instantiate(Store.new(engine), func_reexport_module)
      linker = nil # rubocop:disable Lint/UselessAssignment

      # At this point, we only hold the instance, but the extension should
      # keep the proc and store from being GC'd, so calling should still work
      instance.invoke("f", [])
    end

    private

    def new_linker
      Linker.new(engine)
    end

    def func_reexport_module
      Module.new(engine, <<~WAT)
        (module
          (import "" "" (func))
          (export "f" (func 0)))
      WAT
    end
  end
end
