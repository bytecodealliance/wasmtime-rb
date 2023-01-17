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
        store = Store.new(engine, data)
        10.times { data[:baz] = SecureRandom.hex(1024) }
        data[:baz] = "qux"
        4.times { GC.start(full_mark: true) }
        GC.compact
        expect(store.data).to eql({foo: "bar", baz: "qux"})
      end
    end
  end
end
