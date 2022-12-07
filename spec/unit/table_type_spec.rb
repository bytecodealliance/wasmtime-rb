require "spec_helper"

module Wasmtime
  RSpec.describe TableType do
    it "creates a table type with min and max" do
      type = TableType.new(:funcref, 1, 2)
      expect(type.element).to eq(:funcref)
      expect(type.minimum).to eq(1)
      expect(type.maximum).to eq(2)
    end

    it "creates a table type without maximum" do
      type = TableType.new(:funcref, 1)
      expect(type.minimum).to eq(1)
      expect(type.maximum).to be_nil
    end

    it "raises on invalid Wasm type" do
      expect { TableType.new(:nope, 1) }
        .to raise_error(Wasmtime::Error, /invalid WebAssembly type/)
    end
  end
end
