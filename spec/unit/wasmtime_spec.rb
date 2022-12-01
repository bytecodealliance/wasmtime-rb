require "spec_helper"

module Wasmtime
  RSpec.describe Wasmtime do
    describe ".wat2wasm" do
      it "returns a binary string" do
        wasm = Wasmtime.wat2wasm("(module)")
        expect(wasm.encoding).to eq(Encoding::ASCII_8BIT)
      end

      it "returns a valid module" do
        wasm = Wasmtime.wat2wasm("(module)")
        expect(wasm).to start_with("\x00asm")
      end

      it "raises on invalid WAT" do
        expect { Wasmtime.wat2wasm("not wat") }.to raise_error(Wasmtime::Error)
      end
    end
  end
end
