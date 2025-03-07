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
        [:parallel_compilation, true],
        [:wasm_reference_types, true],
        [:async_stack_zeroing, true]
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
        [:strategy, [:auto, :cranelift, :winch]],
        [:cranelift_opt_level, [:none, :speed, :speed_and_size]],
        [:profiler, profiler_options]
      ].each do |option, valid|
        it "supports #{option}" do
          valid.each { |value| Engine.new(option => value) }
          expect { Engine.new(option => :nope) }
            .to raise_error(ArgumentError, /invalid :#{option}.*:nope/)
        end
      end

      it "supports allocation_strategy config" do
        expect(Engine.new(allocation_strategy: :pooling)).to be_a(Engine)
        expect(Engine.new(allocation_strategy: :on_demand)).to be_a(Engine)
        expect(Engine.new(allocation_strategy: PoolingAllocationConfig.new)).to be_a(Engine)
        expect { Engine.new(allocation_strategy: :nope) }.to raise_error(ArgumentError, /invalid instance allocation strategy: :nope/)
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

    describe ".precompile_component" do
      it "returns a String" do
        serialized = engine.precompile_component("(component)")
        expect(serialized).to be_instance_of(String)
      end

      it "can be used by Component.deserialize" do
        serialized = engine.precompile_component("(component)")
        component = Component::Component.deserialize(engine, serialized)
        expect(component).to be_instance_of(Component::Component)
      end
    end

    describe "#precompile_compatibility_key" do
      it "is the same amongst similar engines" do
        engine_one = Engine.new(target: "x86_64-unknown-linux-gnu", parallel_compilation: true)
        engine_two = Engine.new(target: "x86_64-unknown-linux-gnu", parallel_compilation: false)

        expect(engine_one.precompile_compatibility_key).to eq(engine_two.precompile_compatibility_key)
      end

      it "is different amongst different engines" do
        engine_one = Engine.new(target: "x86_64-unknown-linux-gnu")
        engine_two = Engine.new(target: "arm64-apple-darwin")

        expect(engine_one.precompile_compatibility_key).not_to eq(engine_two.precompile_compatibility_key)
      end

      it "freezes and caches the result to avoid repeated allocation" do
        engine = Engine.new(target: "x86_64-unknown-linux-gnu")

        expect(engine.precompile_compatibility_key).to be_frozen
        expect(engine.precompile_compatibility_key.object_id).to eq(engine.precompile_compatibility_key.object_id)
      end
    end
  end
end
