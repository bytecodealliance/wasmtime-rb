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

      context "limits" do
        [
          :memory_size,
          :table_elements,
          :instances,
          :tables,
          :memories
        ].each do |limit_prop|
          it "rejects non-numeric #{limit_prop}" do
            expect { Store.new(engine, limits: {limit_prop => "bad"}) }.to raise_error(TypeError)
          end
        end

        it "sets a memory size limit" do
          store = Store.new(engine, limits: {memory_size: 150_000})

          mem = Memory.new(store, min_size: 1)
          mem.grow(1)
          expect { mem.grow(1) }.to raise_error(Wasmtime::Error, "failed to grow memory by `1`")
        end

        it "sets a table elements limit" do
          store = Store.new(engine, limits: {table_elements: 1})

          table = Table.new(store, :funcref, nil, min_size: 1)
          expect { table.grow(1, nil) }.to raise_error(Wasmtime::Error, "failed to grow table by `1`")
        end

        it "sets a instances limit" do
          store = Store.new(engine, limits: {instances: 1})

          mod = Module.new(engine, <<~WAT)
            (module
              (func nop)
              (start 0))
          WAT

          Instance.new(store, mod)
          expect { Instance.new(store, mod) }.to raise_error(Wasmtime::Error, "resource limit exceeded: instance count too high at 2")
        end

        it "sets a tables limit" do
          store = Store.new(engine, limits: {tables: 1})

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
          store = Store.new(engine, limits: {memories: 1})

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
          store = Store.new(engine, limits: {memories: 1, tables: 1})

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

      describe "#max_linear_memory_consumed" do
        it "returns the maximum linear memory consumed" do
          store = Store.new(engine, limits: { memory_size: 1_000_000 })
          mod = Module.new(engine, "(module (memory 1) (func (export \"grow\") (param i32) (result i32) (memory.grow (local.get 0))))")
          instance = Instance.new(store, mod)
          grow_func = instance.exports.grow

          grow_func.call(1)
          grow_func.call(2)

          expect(store.max_linear_memory_consumed).to be >= 196608 # 3 pages (64KB each)
        end
      end

      describe "#linear_memory_limit_hit?" do
        it "returns false when the limit is not hit" do
          store = Store.new(engine, limits: { memory_size: 1_000_000 })
          mod = Module.new(engine, "(module (memory 1))")
          Instance.new(store, mod)

          expect(store.linear_memory_limit_hit?).to be false
        end

        it "returns true when the limit is hit" do
          store = Store.new(engine, limits: { memory_size: 65536 }) # 1 page
          mod = Module.new(engine, "(module (memory 1) (func (export \"grow\") (param i32) (result i32) (memory.grow (local.get 0))))")
          instance = Instance.new(store, mod)
          grow_func = instance.exports.grow

          grow_func.call(1) # This should hit the limit

          expect(store.linear_memory_limit_hit?).to be true
        end
      end
    end
  end
end
