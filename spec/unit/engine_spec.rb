require "spec_helper"

module Wasmtime
  RSpec.describe Engine do
    let(:engine) { Engine.new(Wasmtime::Config.new) }

    describe(".precompile_module") do
      it("returns a String") do
        serialized = engine.precompile_module("(module)")
        expect(serialized).to be_instance_of(String)
      end

      it("can be used by Module.deserialize") do
        serialized = engine.precompile_module("(module)")
        mod = Module.deserialize(engine, serialized)
        expect(mod).to be_instance_of(Wasmtime::Module)
      end
    end
  end
end
