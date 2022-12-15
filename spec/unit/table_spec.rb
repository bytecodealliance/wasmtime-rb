require "spec_helper"

module Wasmtime
  RSpec.describe Table do
    describe ".new" do
      it "creates a table with no default" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table).to be_instance_of(Wasmtime::Table)
      end

      it "creates a table with default func" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        expect(table).to be_instance_of(Wasmtime::Table)
      end
    end

    describe "#type" do
      it "returns the Wasm type as a symbol" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.type).to eq(:funcref)
      end
    end

    describe "#min_size" do
      it "returns its min size" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.min_size).to eq(1)
      end
    end

    describe "#max_size" do
      it "returns its max size" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.max_size).to be_nil

        table = Table.new(store, :funcref, nil, min_size: 1, max_size: 2)
        expect(table.max_size).to eq(2)
      end
    end

    describe "#size" do
      it "returns its size" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.size).to eq(1)
      end
    end

    describe "#grow" do
      it "increases the size" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect { table.grow(2, nil) }.to change { table.size }.by(2)
      end

      it "returns the previous size" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.grow(1, nil)).to eq(1)
      end

      it "raises when growing past the maximum" do
        table = Table.new(store, :funcref, nil, min_size: 1, max_size: 1)
        expect { table.grow(1, nil) }.to raise_error(Wasmtime::Error, "failed to grow table by `1`")
      end
    end

    describe "#get" do
      it "returns a Func" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        expect(table.get(0)).to be_instance_of(Func)
      end

      it "returns an externref" do
        value = BasicObject.new
        table = Table.new(store, :externref, value, min_size: 1)
        expect(table.get(0)).to eq(value)
      end

      it "returns nil for null ref" do
        table = Table.new(store, :funcref, nil, min_size: 1)
        expect(table.get(0)).to be_nil
      end

      it "returns nil for out of bound" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        expect(table.get(5)).to be_nil
      end
    end

    describe "#set" do
      it "writes nil" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        table.set(0, nil)
        expect(table.get(0)).to be_nil
      end

      it "writes a Func" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        table.set(0, noop_func)
        expect(table.get(0)).to be_instance_of(Func)
      end

      it "rejects invalid type" do
        table = Table.new(store, :funcref, noop_func, min_size: 1)
        expect { table.set(0, 1) }.to raise_error(TypeError)
      end
    end

    it "keeps externrefs alive" do
      table = Table.new(store, :externref, +"foo", min_size: 2)
      generate_new_objects
      expect(table.get(0)).to eq("foo")

      table.set(1, +"bar")
      generate_new_objects
      expect(table.get(1)).to eq("bar")

      table.grow(1, +"baz")
      generate_new_objects
      expect(table.get(2)).to eq("baz")
    end

    private

    def noop_func
      Func.new(store, [], []) { |_| }
    end

    def generate_new_objects
      "hi" * 3
    end
  end
end
