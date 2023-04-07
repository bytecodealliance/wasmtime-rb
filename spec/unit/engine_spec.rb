require "spec_helper"

module Wasmtime
  RSpec.describe Engine do
    describe ".new" do
      it("accepts a config Hash") { Engine.new(consume_fuel: true) }
      it("accepts no config") { Engine.new }
      it("accepts nil config") { Engine.new(nil) }
      it "rejects non-hash config" do
        expect { Engine.new(1) }.to raise_error(TypeError)
      end
      it "rejects unknown options" do
        expect { Engine.new(nope: 1) }.to raise_error(ArgumentError, "Unknown option: :nope")
      end
      it "rejects multiple args" do
        expect { Engine.new(1, 2) }.to raise_error(ArgumentError)
      end

      # bool & numeric options
      [
        [:debug_info, true],
        [:wasm_backtrace_details, true],
        [:native_unwind_info, true],
        [:consume_fuel, true],
        [:epoch_interruption, true],
        [:max_wasm_stack, 400, true],
        [:wasm_threads, true],
        [:wasm_multi_memory, true],
        [:wasm_memory64, true],
        [:parallel_compilation, true]
      ].each do |option, valid, invalid = nil|
        it "supports #{option}" do
          Engine.new(option => valid)
          expect { Engine.new(option => invalid) }.to raise_error(TypeError, /#{option}/) if invalid
        end
      end

      profiler_options = [:none]
      profiler_options.push(:jitdump, :vtune) if Gem::Platform.local.os == "linux"

      # enum options represented as symbols
      [
        [:cranelift_opt_level, [:none, :speed, :speed_and_size]],
        [:profiler, profiler_options]
      ].each do |option, valid|
        it "supports #{option}" do
          valid.each { |value| Engine.new(option => value) }
          expect { Engine.new(option => :nope) }
            .to raise_error(ArgumentError, /invalid :#{option}.*:nope/)
        end
      end

      it "supports target options" do
        expect { Engine.new(target: "x86_64-unknown-linux-gnu") }.not_to raise_error
        expect { Engine.new(target: "nope") }.to raise_error(ArgumentError, /Unrecognized architecture/)
      end
    end

    describe ".precompile_module" do
      it "returns a String" do
        serialized = engine.precompile_module("(module)")
        expect(serialized).to be_instance_of(String)
      end

      it "can be used by Module.deserialize" do
        serialized = engine.precompile_module("(module)")
        mod = Module.deserialize(engine, serialized)
        expect(mod).to be_instance_of(Wasmtime::Module)
      end
    end
  end
end
