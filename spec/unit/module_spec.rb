require "spec_helper"
require "tempfile"

module Wasmtime
  RSpec.describe Module do
    let(:engine) { Engine.new(Wasmtime::Config.new) }

    it("can be serialized and deserialized") do
      mod = Module.new(engine, wat)
      serialized = mod.serialize
      deserialized = Module.deserialize(engine, serialized)
      expect(deserialized.serialize).to eq(serialized)
    end

    describe(".deserialize_file") do
      it("can deserialize a module from a file") do
        tmpfile = Tempfile.new(["deserialize-file", ".so"])
        tmpfile.write(Module.new(engine, "(module)").serialize)
        tmpfile.flush

        begin
          mod = Module.deserialize_file(engine, tmpfile.path)
          expect(mod.serialize).to eq(Module.new(engine, "(module)").serialize)
        ensure
          tmpfile.close
        end
      end

      it("deserialize from a module multiple times") do
        tmpfile = Tempfile.new(["deserialize-file", ".so"])
        tmpfile.write(Module.new(engine, wat).serialize)
        tmpfile.flush

        begin
          mod_one = Module.deserialize_file(engine, tmpfile.path)
          mod_two = Module.deserialize_file(engine, tmpfile.path)
          expected = Module.new(engine, wat).serialize

          expect(mod_one.serialize).to eq(expected)
          expect(mod_two.serialize).to eq(expected)
        ensure
          tmpfile.close
        end
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
