require "spec_helper"

module Wasmtime
  RSpec.describe Memory do
    it "creates a memory" do
      mem = Memory.new(store, MemoryType.new(1))
      expect(mem).to be_instance_of(Wasmtime::Memory)
    end

    it "returns its size" do
      mem = Memory.new(store, MemoryType.new(1))
      expect(mem.size).to eq(1)
    end

    it "returns its memory type" do
      mem = Memory.new(store, MemoryType.new(1))
      expect(mem.ty).to be_instance_of(MemoryType)
      expect(mem.ty.minimum).to eq(1)
      expect(mem.ty.maximum).to be_nil
    end

    it "returns the previous size when growing" do
      mem = Memory.new(store, MemoryType.new(2))
      expect(mem.grow(1)).to eq(2)
    end

    it "rejects growing past its maximum" do
      mem = Memory.new(store, MemoryType.new(1, 1))
      expect { mem.grow(1) }.to raise_error(Wasmtime::Error, "failed to grow memory by `1`")
    end

    it "reads and writes a string" do
      mem = Memory.new(store, MemoryType.new(1))
      expect(mem.write(0, "foo")).to be_nil
      expect(mem.read(0, 3)).to eq("foo")
    end

    it "raises when writing past the end of the buffer" do
      mem = Memory.new(store, MemoryType.new(1))
      expect { mem.write(64 * 2**10, "f") }
        .to raise_error(Wasmtime::Error, "out of bounds memory access")
    end

    it "raises when reading past the end of the buffer" do
      mem = Memory.new(store, MemoryType.new(1))
      expect { mem.read(64 * 2**10, 1) }
        .to raise_error(Wasmtime::Error, "out of bounds memory access")
    end
  end
end
