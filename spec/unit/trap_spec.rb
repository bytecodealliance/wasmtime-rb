require "spec_helper"

module Wasmtime
  RSpec.describe Trap do
    let(:trap) do
      Wasmtime::Instance.new(store, module_trapping_on_start)
    rescue Trap => trap
      trap
    end

    describe "#message" do
      it "has a short message" do
        expect(trap.message).to eq("wasm trap: wasm `unreachable` instruction executed")
      end
    end

    describe "#message_with_backtrace" do
      it "includes the backtrace" do
        expect(trap.wasm_backtrace_message).to eq(<<~MSG.rstrip)
          error while executing at wasm backtrace:
              0:     0x1a - <unknown>!<wasm function 0>
        MSG
      end
    end

    describe "#wasm_backtrace" do
      it "returns an enumerable of trace entries" do
      end
    end

    describe "#code" do
      it "returns a symbol matching a constant" do
        expect(trap.code).to eq(Trap::UNREACHABLE_CODE_REACHED)
      end
    end

    describe "#to_s" do
      it "is the same as message" do
        expect(trap.to_s).to eq(trap.message)
      end
    end

    describe "#inspect" do
      it "looks pretty" do
        expect(trap.inspect).to match(/\A#<Wasmtime::Trap:0x\h{16} @trap_code=:unreachable_code_reached>$/)
      end
    end

    def module_trapping_on_start
      Wasmtime::Module.new(engine, <<~WAT)
        (module
          (func unreachable)
          (start 0))
      WAT
    end
  end
end
