require "spec_helper"
require "fiddle"

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

      fit "tracks memory usage" do
        wasm_page_size = 0x10000
        mem = Memory.new(store, min_size: 1, max_size: 4)
        _, increase_bytes = measure_gc_stat(:malloc_increase_bytes) { mem.grow(3) }
        expect(increase_bytes).to be >= (3 * wasm_page_size)
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

    describe "#read_utf8" do
      it "reads a UTF-8 string" do
        mem = Memory.new(store, min_size: 1)
        expect(mem.write(0, "foo")).to be_nil
        str = mem.read_utf8(0, 3)
        expect(str).to eq("foo")
        expect(str.encoding).to eq(Encoding::UTF_8)
      end

      it "raises when the utf8 is invalid" do
        invalid_utf8 = [0x80, 0x80, 0x80].pack("C*")
        mem = Memory.new(store, min_size: 1)
        expect(mem.write(0, invalid_utf8)).to be_nil

        expect { mem.read_utf8(0, 3) }.to raise_error(Wasmtime::Error, /invalid utf-8/)
      end
    end

    describe "#unsafe_slice" do
      it "exposes a frozen string" do
        mem = Memory.new(store, min_size: 1)
        mem.write(0, "foo")
        str = String(mem.read_unsafe_slice(0, 3))

        expect(str).to eq("foo")
        expect(str.encoding).to eq(Encoding::ASCII_8BIT)
        expect(str).to be_frozen
      end

      if defined?(Fiddle::MemoryView)
        it "exposes a memory view" do
          mem = Memory.new(store, min_size: 3)
          mem.write(0, "foo")
          view = mem.read_unsafe_slice(0, 3).to_memory_view

          expect(view).to be_a(Fiddle::MemoryView)
          expect(view).to be_readonly
          expect(view.ndim).to eq(1)
          expect(view.to_s).to eq("foo") unless RUBY_VERSION.start_with?("3.0")
        end
      end

      it "invalidates the size when the memory is resized" do
        mem = Memory.new(store, min_size: 1)
        mem.write(0, "foo")
        slice = mem.read_unsafe_slice(0, 3)
        mem.grow(1)

        expect { slice.to_str }
          .to raise_error(Wasmtime::Error, "memory slice was invalidated by resize")

        if defined?(Fiddle::MemoryView)
          expect { slice.to_memory_view }
            .to raise_error(ArgumentError, /Unable to get a memory view from/)
        end
      end

      it "errors when the memory is out of bounds" do
        mem = Memory.new(store, min_size: 1)

        expect { mem.read_unsafe_slice(64 * 2**10, 1) }
          .to raise_error(Wasmtime::Error, "out of bounds memory access")
      end
    end
  end
end
