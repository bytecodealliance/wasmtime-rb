require "spec_helper"
require "tempfile"
require "securerandom"
require "pathname"

module Wasmtime
  RSpec.describe Module do
    let(:engine) { Engine.new(Wasmtime::Config.new) }

    it("can be serialized and deserialized") do
      mod = Module.new(engine, wat)
      serialized = mod.serialize
      deserialized = Module.deserialize(engine, serialized)
      expect(deserialized.serialize).to eq(serialized)
    end

    describe(".from_file") do
      it("loads the module") do
        mod = Module.from_file(engine, "spec/fixtures/empty.wat")
        expect(mod).to be_instance_of(Module)
      end
    end

    describe(".deserialize_file") do
      include_context(:tmpdir)

      it("can deserialize a module from a file") do
        tmpfile = create_tmpfile(Module.new(engine, "(module)").serialize)
        mod = Module.deserialize_file(engine, tmpfile)

        expect(mod.serialize).to eq(Module.new(engine, "(module)").serialize)
      end

      it("deserialize from a module multiple times") do
        tmpfile = create_tmpfile(Module.new(engine, wat).serialize)

        mod_one = Module.deserialize_file(engine, tmpfile)
        mod_two = Module.deserialize_file(engine, tmpfile)
        expected = Module.new(engine, wat).serialize

        expect(mod_one.serialize).to eq(expected)
        expect(mod_two.serialize).to eq(expected)
      end

      def create_tmpfile(content)
        uuid = SecureRandom.uuid
        path = File.join(tmpdir, "deserialize-file-test-#{uuid}.so")
        File.binwrite(path, content)
        path
      end
    end

    describe(".deserialize") do
      it("raises on invalid module") do
        expect { Module.deserialize(engine, "foo") }
          .to raise_error(Wasmtime::Error)
      end
    end
  end
end
