require "spec_helper"

module Wasmtime
  RSpec.describe Linker do
    it "#allow_shadowing" do
      linker = new_linker
      linker.func_new("foo", "bar", [], []) {}

      linker.allow_shadowing = false
      expect { linker.func_new("foo", "bar", [], []) {} }
        .to raise_error(Wasmtime::Error)

      linker.allow_shadowing = true
      expect { linker.func_new("foo", "bar", [], []) {} }
        .not_to raise_error
    end

    it "#allow_unknown_exports" do
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

    it "#define_unknown_imports_as_traps" do
      mod = Module.new(engine, '(module (import "" "" (func)))')
      linker = new_linker
      expect { linker.instantiate(store, mod) }.to raise_error(Wasmtime::Error, /unknown import/)

      linker.define_unknown_imports_as_traps(mod)
      expect { linker.instantiate(store, mod) }.not_to raise_error
    end

    describe "#define" do
      it "accepts memory" do
        linker = new_linker
        store = Store.new(engine)
        memory = Memory.new(store, min_size: 1)
        linker.define(store, "mod", "mem", memory)
        expect(linker.get(store, "mod", "mem").to_memory).to be_instance_of(Memory)
      end

      it "accepts func" do
        linker = new_linker
        store = Store.new(engine)
        func = Func.new(store, [], []) {}
        linker.define(store, "mod", "fn", func)
        expect(linker.get(store, "mod", "fn").to_func).to be_instance_of(Func)
      end

      it "accepts table" do
        linker = new_linker
        table = Table.new(store, :funcref, nil, min_size: 1)
        linker.define(store, "mod", "table", table)
        expect(linker.get(store, "mod", "table").to_table).to be_instance_of(Table)
      end

      it "accepts global" do
        linker = new_linker
        store = Store.new(engine)
        global = Global.var(store, :i32, 1)
        linker.define(store, "mod", "glob", global)
        expect(linker.get(store, "mod", "glob").to_global).to be_instance_of(Global)
      end
    end

    describe "func_new" do
      it "requires a block" do
        expect { new_linker.func_new("foo", "bar", [], []) }
          .to raise_error(ArgumentError, "no block given")

        expect { new_linker.func_new("foo", "bar", [], []) {} }
          .not_to raise_error
      end

      it "registers a Func that can be called" do
        calls = 0
        linker = new_linker
        linker.func_new("", "", [], []) do |caller|
          calls += 1
          expect(caller).to be_instance_of(Caller)
        end
        func = linker.get(Store.new(engine), "", "").to_func
        expect { func.call }.to change { calls }.by(1)
      end
    end

    describe "#get" do
      it "returns nil for undefined items" do
        linker = new_linker
        store = Store.new(engine)
        expect(linker.get(store, "nope", "nope")).to be_nil
      end

      it "returns an Extern" do
        linker = new_linker
        linker.func_new("mod", "fn", [], [:i32]) { 42 }
        extern = linker.get(Store.new(engine), "mod", "fn")
        expect(extern).to be_instance_of(Extern)
      end
    end

    it "#instance" do
      linker = new_linker
      mod = Module.new(engine, '(module (func (export "fn")))')
      linker.instance(store, "mod", Wasmtime::Instance.new(store, mod))
      expect(linker.get(store, "mod", "fn")).to be_truthy
    end

    it "#module" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      expect(linker.get(store, "mod1", "fn1")).to be_truthy
    end

    it "#alias" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      linker.alias("mod1", "fn1", "mod2", "fn2")
      expect(linker.get(store, "mod2", "fn2")).to be_truthy
    end

    it "#alias_module" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "fn1")))'))
      linker.alias_module("mod1", "mod2")
      expect(linker.get(store, "mod2", "fn1")).to be_truthy
    end

    it "#instantiate" do
      linker = new_linker
      linker.func_new("", "", [], []) {}
      instance = linker.instantiate(Store.new(engine), func_reexport_module)
      expect(instance).to be_instance_of(Instance)
    end

    it "#get_default" do
      linker = new_linker
      store = Store.new(engine)
      linker.module(store, "mod1", Module.new(engine, '(module (func (export "")))'))
      linker.module(store, "mod2", Module.new(engine, '(module (memory (export "") 1))'))

      expect(linker.get_default(store, "mod1")).to be_instance_of(Func)
      expect { linker.get_default(store, "mod2") }.to raise_error(Wasmtime::Error, /not a function/)
    end

    it "#use_deterministic_scheduling_functions" do
      linker = new_linker
      # The call should succeed as opposed to raising an error
      # Can't really test this without relying on internal details of
      # `deterministic-wasi-ctx`
      linker.use_deterministic_scheduling_functions
    end

    it "GC stresses instance and func" do
      calls = 0
      linker = new_linker
      linker.func_new("", "", [], []) { calls += 1 }
      instance = linker.instantiate(Store.new(engine), func_reexport_module)
      linker = nil # rubocop:disable Lint/UselessAssignment

      # At this point, we only hold the instance, but the extension should
      # keep the proc and store from being GC'd, so calling should still work
      instance.invoke("f")
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
