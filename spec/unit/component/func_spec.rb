require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Func do
      before(:all) do
        @adder_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_adder.wat")
        @trap_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_trap.wat")
      end

      let(:linker) { Linker.new(engine) }
      let(:add) { linker.instantiate(store, @adder_component).get_func("add") }
      let(:unreachable) { linker.instantiate(store, @trap_component).get_func("unreachable") }

      describe "#call" do
        it "calls the func" do
          expect(add.call(1, 2)).to eq(3)
        end

        it "allows multiple calls into the same component instance" do
          expect(add.call(1, 2)).to eq(3)
          expect(add.call(1, 2)).to eq(3)
        end

        it "raises on invalid arg count" do
          expect { add.call(1) }
            .to raise_error(ArgumentError, /(given 1, expected 2)/)
        end

        it "raises on invalid arg type" do
          expect { add.call(nil, nil) }
            .to raise_error(TypeError, "no implicit conversion of nil into Integer (param at index 0)")
        end

        it "raises trap when component traps" do
          expect { unreachable.call }.to raise_error(Trap) do |trap|
            expect(trap.code).to eq(Trap::UNREACHABLE_CODE_REACHED)
          end
        end
      end
    end
  end
end
