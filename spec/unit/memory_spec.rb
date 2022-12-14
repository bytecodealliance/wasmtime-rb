require "spec_helper"

module Wasmtime
  RSpec.describe Memory do
    describe ".new" do
      it "creates a memory" do
        mem = Memory.new(store, min_size: 1)
        expect(mem).to be_instance_of(Wasmtime::Memory)
      end
    end

    describe "#size" do
      it "returns its size" do
        mem = Memory.new(store, min_size: 1)
        expect(mem.size).to eq(1)
      end
    end

    describe "#min_size" do
      it "returns the memory type's min size" do
        mem = Memory.new(store, min_size: 1)
        expect(mem.min_size).to eq(1)
      end
    end

    describe "#max_size" do
      it "defaults to nil" do
        mem = Memory.new(store, min_size: 1)
        expect(mem.max_size).to be_nil
      end

      it "returns the memory type's max size" do
        mem = Memory.new(store, min_size: 1, max_size: 2)
        expect(mem.max_size).to eq(2)
      end
    end

    describe "#grow" do
      it "returns the previous size" do
        mem = Memory.new(store, min_size: 2)
        expect(mem.grow(1)).to eq(2)
      end

      it "raises when growing past the maximum" do
        mem = Memory.new(store, min_size: 1, max_size: 1)
        expect { mem.grow(1) }.to raise_error(Wasmtime::Error, "failed to grow memory by `1`")
      end
    end

    describe "#read, #write" do
      it "reads and writes a Binary string" do
        mem = Memory.new(store, min_size: 1)
        expect(mem.write(0, "foo")).to be_nil
        str = mem.read(0, 3)
        expect(str).to eq("foo")
        expect(str.encoding).to eq(Encoding::ASCII_8BIT)
      end

      it "raises when reading past the end of the buffer" do
        mem = Memory.new(store, min_size: 1)
        expect { mem.read(64 * 2**10, 1) }
          .to raise_error(Wasmtime::Error, "out of bounds memory access")
      end

      it "raises when writing past the end of the buffer" do
        mem = Memory.new(store, min_size: 1)
        expect { mem.write(64 * 2**10, "f") }
          .to raise_error(Wasmtime::Error, "out of bounds memory access")
      end
    end
  end
end
