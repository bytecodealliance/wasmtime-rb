require "spec_helper"
require "tempfile"
require "securerandom"
require "pathname"

module Wasmtime
  RSpec.describe Module do
    it "can be serialized and deserialized" do
      mod = Module.new(engine, wat)
      serialized = mod.serialize
      deserialized = Module.deserialize(engine, serialized)
      expect(deserialized.serialize).to eq(serialized)
    end

    describe ".from_file" do
      it "loads the module" do
        mod = Module.from_file(engine, "spec/fixtures/empty.wat")
        expect(mod).to be_instance_of(Module)
      end

      it "tracks memory usage" do
        before = GC.stat(:malloc_increase_bytes)
        Module.from_file(engine, "spec/fixtures/empty.wat")
        after = GC.stat(:malloc_increase_bytes)

        # This is a rough estimate of the memory usage of the module, subject to compiler changes
        expect(2500..2700).to include(after - before)
      end
    end

    describe ".deserialize_file" do
      include_context(:tmpdir)
      let(:tmpdir) { Dir.mktmpdir }

      after(:each) do
        FileUtils.rm_rf(tmpdir)
      rescue Errno::EACCES => e
        warn "WARN: Failed to remove #{tmpdir} (#{e})"
      end

      it("can deserialize a module from a file") do
        tmpfile = create_tmpfile(Module.new(engine, "(module)").serialize)
        mod = Module.deserialize_file(engine, tmpfile)

        expect(mod.serialize).to eq(Module.new(engine, "(module)").serialize)
      end

      it "deserialize from a module multiple times" do
        tmpfile = create_tmpfile(Module.new(engine, wat).serialize)

        mod_one = Module.deserialize_file(engine, tmpfile)
        mod_two = Module.deserialize_file(engine, tmpfile)
        expected = Module.new(engine, wat).serialize

        expect(mod_one.serialize).to eq(expected)
        expect(mod_two.serialize).to eq(expected)
      end

      it "tracks memory usage" do
        tmpfile = create_tmpfile(Module.new(engine, "(module)").serialize)
        before = GC.stat(:malloc_increase_bytes)
        Module.deserialize_file(engine, tmpfile)
        after = GC.stat(:malloc_increase_bytes)

        expect(after - before).to equal(File.size(tmpfile))
      end

      def create_tmpfile(content)
        uuid = SecureRandom.uuid
        path = File.join(tmpdir, "deserialize-file-test-#{uuid}.so")
        File.binwrite(path, content)
        path
      end
    end

    describe ".deserialize" do
      it "raises on invalid module" do
        expect { Module.deserialize(engine, "foo") }
          .to raise_error(Wasmtime::Error)
      end

      it "tracks memory usage" do
        serialized = Module.new(engine, wat).serialize
        before = GC.stat(:malloc_increase_bytes)
        Module.deserialize(engine, serialized)
        after = GC.stat(:malloc_increase_bytes)

        expect(after - before).to equal(serialized.bytesize)
      end
    end
  end
end
