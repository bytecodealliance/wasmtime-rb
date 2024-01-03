require "spec_helper"

module Wasmtime
  RSpec.describe "Fuel" do
    def self.test_on_store_and_caller(desc, store = :store, &block)
      context "on Store" do
        it desc do
          instance_exec(send(store), &block)
        end
      end

      context "on Caller" do
        it desc do
          func = Func.new(send(store), [], []) do |caller|
            instance_exec(caller, &block)
          end
          func.call
        end
      end
    end

    let(:engine) { Engine.new(consume_fuel: true) }
    let(:store) { Store.new(engine) }
    let(:store_without_fuel) { Store.new(Engine.new) }

    describe "#set_fuel" do
      test_on_store_and_caller "returns nil on success" do |store_like|
        expect(store_like.set_fuel(100)).to be_nil
      end

      test_on_store_and_caller "raises when fuel isn't configured", :store_without_fuel do |store_like|
        expect { store_like.set_fuel(100) }
          .to(raise_error(Wasmtime::Error, /fuel is not configured in this store/))
      end
    end

    describe "#get_fuel" do
      test_on_store_and_caller "starts at 0" do |store_like|
        expect(store_like.get_fuel).to eq(0)
      end

      test_on_store_and_caller "raises an error when fuel is not configured", :store_without_fuel do |store_like|
        expect { store_like.get_fuel }.to(raise_error(Wasmtime::Error, /fuel is not configured in this store/))
      end
    end

    it "traps when Wasm execution runs out of fuel" do
      mod = Module.new(engine, <<~WAT)
        (module
          (func (export "f") (result i32)
            i32.const 42))
      WAT
      instance = Instance.new(store, mod)
      store.set_fuel(1)
      expect { instance.invoke("f") }.to raise_error(Trap, /all fuel consumed/) do |error|
        expect(error.code).to eq(Trap::OUT_OF_FUEL)
      end
    end
  end
end
