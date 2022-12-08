require "spec_helper"

module Wasmtime
  RSpec.describe Global do
    describe ".new" do
      it "creates a global" do
        global = Global.new(store, GlobalType.const(:i32), 1)
        expect(global).to be_instance_of(Wasmtime::Global)
      end
    end

    describe "#get" do
      it "returns the global value" do
        global = Global.new(store, GlobalType.const(:i32), 1)
        expect(global.get).to eq(1)
      end
    end

    describe "#set" do
      it "changes the value" do
        global = Global.new(store, GlobalType.var(:i32), 1)
        global.set(2)
        expect(global.get).to eq(2)
      end

      it "raises when the global is constant" do
        global = Global.new(store, GlobalType.const(:i32), 1)
        expect { global.set(2) }
          .to raise_error(Wasmtime::Error, "immutable global cannot be set")
      end
    end

    describe "#ty" do
      it "returns the global type" do
        ty = Global.new(store, GlobalType.var(:i32), 1).ty
        expect(ty).to be_instance_of(GlobalType)
        expect(ty).to be_var
        expect(ty.content).to eq(:i32)
      end
    end

    it "keeps externrefs alive" do
      global = Global.new(store, GlobalType.var(:externref), +"foo")
      generate_new_objects
      expect(global.get).to eq("foo")

      global.set(+"bar")
      generate_new_objects
      expect(global.get).to eq("bar")
    end

    private

    def generate_new_objects
      "hi" * 3
    end
  end
end
