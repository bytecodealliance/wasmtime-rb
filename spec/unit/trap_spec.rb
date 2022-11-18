require "spec_helper"

module Wasmtime
  RSpec.describe Trap do
    let(:trap) do
      Wasmtime::Instance.new(store, module_trapping_on_start)
    rescue Trap => trap
      trap
    end

    describe "#message" do
      it "has the full message including backtrace" do
        expect(trap.message).to eq(<<~MSG)
          wasm trap: wasm `unreachable` instruction executed
          wasm backtrace:
              0:   0x1a - <unknown>!<wasm function 0>
        MSG
      end
    end

    describe "#trap_code" do
      it "returns a symbol matching a constant" do
        expect(trap.trap_code).to eq(TrapCode::UNREACHABLE_CODE_REACHED)
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
