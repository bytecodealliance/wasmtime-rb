require "spec_helper"

module Wasmtime
  RSpec.describe Table do
    describe ".new" do
      it "creates a table with no default" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect(table).to be_instance_of(Wasmtime::Table)
      end

      it "creates a table with default func" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        expect(table).to be_instance_of(Wasmtime::Table)
      end
    end

    describe "#size" do
      it "returns its size" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect(table.size).to eq(1)
      end
    end

    describe "#ty" do
      it "returns its table type" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect(table.ty).to be_instance_of(TableType)
        expect(table.ty.minimum).to eq(1)
        expect(table.ty.maximum).to be_nil
      end
    end

    describe "#grow" do
      it "increases the size" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect { table.grow(2, nil) }.to change { table.size }.by(2)
      end

      it "returns the previous size" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect(table.grow(1, nil)).to eq(1)
      end

      it "raises when growing past the maximum" do
        table = Table.new(store, TableType.new(:funcref, 1, 1), nil)
        expect { table.grow(1, nil) }.to raise_error(Wasmtime::Error, "failed to grow table by `1`")
      end
    end

    describe "#get" do
      it "returns a Func" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        expect(table.get(0)).to be_instance_of(Func)
      end

      it "returns an externref" do
        value = BasicObject.new
        table = Table.new(store, TableType.new(:externref, 1), value)
        expect(table.get(0)).to eq(value)
      end

      it "returns nil for null ref" do
        table = Table.new(store, TableType.new(:funcref, 1), nil)
        expect(table.get(0)).to be_nil
      end

      it "returns nil for out of bound" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        expect(table.get(5)).to be_nil
      end
    end

    describe "#set" do
      it "writes nil" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        table.set(0, nil)
        expect(table.get(0)).to be_nil
      end

      it "writes a Func" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        table.set(0, noop_func)
        expect(table.get(0)).to be_instance_of(Func)
      end

      it "rejects invalid type" do
        table = Table.new(store, TableType.new(:funcref, 1), noop_func)
        expect { table.set(0, 1) }.to raise_error(TypeError)
      end
    end

    it "keeps externrefs alive" do
      table = Table.new(store, TableType.new(:externref, 2), +"foo")
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
