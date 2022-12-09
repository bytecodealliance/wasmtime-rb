require "spec_helper"

module Wasmtime
  RSpec.describe GlobalType do
    describe ".const" do
      it "creates a const global type" do
        type = GlobalType.const(:i32)
        expect(type).to be_const
        expect(type).not_to be_var
      end

      it "raises on invalid Wasm type" do
        expect { GlobalType.const(:nope) }
          .to raise_error(Wasmtime::Error, /invalid WebAssembly type/)
      end
    end

    describe ".var" do
      it "creates a var global type" do
        type = GlobalType.var(:i32)
        expect(type).to be_var
        expect(type).not_to be_const
      end

      it "raises on invalid Wasm type" do
        expect { GlobalType.var(:nope) }
          .to raise_error(Wasmtime::Error, /invalid WebAssembly type/)
      end
    end

    describe "#content" do
      it "returns the Wasm type as symbol" do
        type = GlobalType.const(:i32)
        expect(type.content).to eq(:i32)
      end
    end
  end
end
