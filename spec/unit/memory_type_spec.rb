require "spec_helper"

module Wasmtime
  RSpec.describe MemoryType do
    it "creates a memory type with min and max" do
      type = MemoryType.new(1, 2)
      expect(type.minimum).to eq(1)
      expect(type.maximum).to eq(2)
    end

    it "creates a memory type without maximum" do
      type = MemoryType.new(1)
      expect(type.minimum).to eq(1)
      expect(type.maximum).to be_nil
    end
  end
end
