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

      it "sets a limit" do
        store = Store.new(engine)
        store.set_limits(memory_size: 150_000)

        mem = Memory.new(store, min_size: 1)
        mem.grow(1)
        expect { mem.grow(1) }.to raise_error(Wasmtime::Error, "failed to grow memory by `1`")
      end
    end
  end
end
