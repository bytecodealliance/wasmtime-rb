require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Instance do
      before(:all) do
        @adder_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_adder.wat")
      end

      let(:linker) { Linker.new(engine) }
      let(:adder_instance) { linker.instantiate(store, @adder_component) }

      describe "#get_func" do
        it "returns a root func" do
          expect(adder_instance.get_func("add")).to be_instance_of(Wasmtime::Component::Func)
        end

        it "returns a nested func" do
          expect(adder_instance.get_func(["adder", "add"])).to be_instance_of(Wasmtime::Component::Func)
        end

        it "returns nil for invalid func" do
          expect(adder_instance.get_func("no")).to be_nil
          expect(adder_instance.get_func(["add", "no"])).to be_nil
        end

        it "raises for invalid arg" do
          expect { adder_instance.get_func(3) }
            .to raise_error(TypeError, /invalid argument for component index/)

          expect { adder_instance.get_func([nil]) }
            .to raise_error(TypeError, /invalid argument for component index/)
        end
      end
    end
  end
end
