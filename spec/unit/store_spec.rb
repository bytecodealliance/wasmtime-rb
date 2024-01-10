require "spec_helper"

module Wasmtime
  RSpec.describe Store do
    describe ".new" do
      it "default to nil data" do
        store = Store.new(engine)
        expect(store.data).to be_nil
      end

      it "accepts user-provided data" do
        data = BasicObject.new
        store = Store.new(engine, data)
        expect(store.data).to equal(data)
      end

      it "can be gc compacted" do
        data = {foo: "bar"}
        10.times { data[:baz] = SecureRandom.hex(1024) }
        obj = Struct.new(:value).new(data)
        store = Store.new(engine, obj)
        10.times { data[:baz] = SecureRandom.hex(1024) }
        data[:baz] = "qux"
        4.times { GC.start(full_mark: true) }
        GC.compact
        expect(store.data.value).to eql({foo: "bar", baz: "qux"})
      end
    end

    describe "#set_limits" do
      it "sets a memory size limit" do
        store = Store.new(engine)
        store.set_limits(memory_size: 150_000)

        mem = Memory.new(store, min_size: 1)
        mem.grow(1)
        expect { mem.grow(1) }.to raise_error(Wasmtime::Error, "failed to grow memory by `1`")
      end

      it "sets a table elements limit" do
        store = Store.new(engine)
        store.set_limits(table_elements: 1)

        table = Table.new(store, :funcref, nil, min_size: 1)
        expect { table.grow(1, nil) }.to raise_error(Wasmtime::Error, "failed to grow table by `1`")
      end

      it "sets a instances limit" do
        store = Store.new(engine)
        store.set_limits(instances: 1)

        mod = Module.new(engine, <<~WAT)
          (module
            (func nop)
            (start 0))
        WAT

        Instance.new(store, mod)
        expect { Instance.new(store, mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: instance count too high at 2")
      end

      it "sets a tables limit" do
        store = Store.new(engine)
        store.set_limits(tables: 1)

        mod = Module.new(engine, <<~WAT)
          (module
            (table $table1 1 funcref)
            (table $table2 1 funcref)
            (func nop)
            (start 0))
        WAT

        expect { Instance.new(store, mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: table count too high at 2")
      end

      it "sets a memories limit" do
        store = Store.new(engine)
        store.set_limits(memories: 1)

        mod = Module.new(engine, <<~WAT)
          (module
            (memory $memory1 1)
            (memory $memory2 1)
            (func nop)
            (start 0))
        WAT

        expect { Instance.new(store, mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: memory count too high at 2")
      end

      it "handles multiple keywords" do
        store = Store.new(engine)
        store.set_limits(memories: 1, tables: 1)

        memory_mod = Module.new(engine, <<~WAT)
          (module
            (memory $memory1 1)
            (memory $memory2 1)
            (func nop)
            (start 0))
        WAT

        table_mod = Module.new(engine, <<~WAT)
          (module
            (table $table1 1 funcref)
            (table $table2 1 funcref)
            (func nop)
            (start 0))
        WAT

        expect { Instance.new(store, memory_mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: memory count too high at 2")
        expect { Instance.new(store, table_mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: table count too high at 2")
      end
    end
  end
end
