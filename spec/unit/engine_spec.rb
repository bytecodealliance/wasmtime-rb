require "spec_helper"

module Wasmtime
  RSpec.describe Engine do
    let(:engine) { Engine.new(Wasmtime::Config.new) }

    describe(".new") do
      it("accepts a config") { Engine.new(Wasmtime::Config.new) }
      it("accepts no config") { Engine.new }
      it("accepts nil config") { Engine.new(nil) }
      it("rejects non-config arg") do
        expect { Engine.new(1) }.to raise_error(TypeError)
      end
      it("rejects multiple args") do
        expect { Engine.new(1, 2) }.to raise_error(ArgumentError)
      end
    end

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
