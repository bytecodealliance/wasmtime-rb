require "spec_helper"

module Wasmtime
  RSpec.describe Global do
    describe ".const" do
      it "creates a const global" do
        global = Global.const(store, :i32, 1)
        expect(global).to be_const
        expect(global).not_to be_var
      end

      it "raises on invalid Wasm type" do
        expect { Global.const(store, :nope, 1) }
          .to raise_error(ArgumentError, /invalid WebAssembly type/)
      end
    end

    describe ".var" do
      it "creates a var global" do
        global = Global.var(store, :i32, 1)
        expect(global).to be_var
        expect(global).not_to be_const
      end

      it "raises on invalid Wasm type" do
        expect { Global.var(store, :nope, 1) }
          .to raise_error(ArgumentError, /invalid WebAssembly type/)
      end
    end

    describe "#type" do
      it "returns the Wasm type as symbol" do
        global = Global.const(store, :i32, 1)
        expect(global.type).to eq(:i32)
      end
    end

    describe "#get" do
      it "returns the global value" do
        global = Global.var(store, :i32, 1)
        expect(global.get).to eq(1)
      end
    end

    describe "#set" do
      it "changes the value" do
        global = Global.var(store, :i32, 1)
        global.set(2)
        expect(global.get).to eq(2)
      end

      it "raises when the global is constant" do
        global = Global.const(store, :i32, 1)
        expect { global.set(2) }
          .to raise_error(Wasmtime::Error, "immutable global cannot be set")
      end
    end

    it "keeps externrefs alive" do
      global = Global.var(store, :externref, +"foo")
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
