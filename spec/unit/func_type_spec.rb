require "spec_helper"

module Wasmtime
  RSpec.describe FuncType do
    it("accepts supported Wasm types") do
      supported_types = [:i32, :i64, :f32, :f64, :v128, :funcref, :externref]
      supported_types.each do |type|
        ty = FuncType.new([type], [])
        expect(ty.params).to eq([type])
        expect(ty.results).to eq([])

        ty = FuncType.new([], [type])
        expect(ty.params).to eq([])
        expect(ty.results).to eq([type])
      end
    end

    it("rejects unknown symbols") do
      expect { FuncType.new([:nope], []) }
        .to raise_error(Wasmtime::Error, /expected one of \[:i32, :i64, :f32, :f64, :v128, :funcref, :externref\], got :nope/)
    end

    it("rejects non-symbols") do
      expect { FuncType.new(nil, nil) }.to raise_error(Wasmtime::Error)
      expect { FuncType.new([1], [2]) }.to raise_error(Wasmtime::Error)
    end
  end
end
