require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Variant do
      it "has name" do
        expect(Variant.new("a", 1).name).to eq("a")
      end

      it "has value" do
        expect(Variant.new("a", 1).value).to eq(1)
      end

      it "behaves like a value object" do
        expect(Variant.new("a", 1)).to eq(Variant.new("a", 1))
        expect(Variant.new("a", 1).hash).to eq(Variant.new("a", 1).hash)

        expect(Variant.new("a")).not_to eq(Variant.new("b"))
        expect(Variant.new("a", 1)).not_to eq(Variant.new("a", 2))
        expect(Variant.new("a").hash).not_to eq(Variant.new("b").hash)
        expect(Variant.new("a", 1).hash).not_to eq(Variant.new("a", 2).hash)
      end
    end
  end
end
