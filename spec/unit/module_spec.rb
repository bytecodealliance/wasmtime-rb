require "spec_helper"

module Wasmtime
  RSpec.describe Module do
    let(:engine) { Engine.new(Wasmtime::Config.new) }

    it("can be serialized and deserialized") do
      mod = Module.new(engine, "(module)")
      serialized = mod.serialize
      deserialized = Module.deserialize(engine, serialized)
      expect(deserialized.serialize).to eq(serialized)
    end

    describe(".deserialize") do
      it("raises on invalid module") do
        expect { Module.deserialize(engine, "foo") }
          .to raise_error(Wasmtime::Error)
      end
    end
  end
end
