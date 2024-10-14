require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Instance do
      before(:all) do
        @adder_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_adder.wat")
      end

      let(:linker) { Linker.new(engine) }
      let(:adder_instance) { linker.instantiate(store, @adder_component) }

      describe "#invoke" do
        it "calls the export" do
          expect(adder_instance.invoke("add", 1, 2)).to eq(3)
        end

        it "allows multiple calls into the same component instance" do
          expect(adder_instance.invoke("add", 1, 2)).to eq(3)
          expect(adder_instance.invoke("add", 1, 2)).to eq(3)
        end

        it "raises on unknown exports" do
          expect { adder_instance.invoke("nope") }
            .to raise_error(Wasmtime::Error, /function "nope" not found/)
        end

        it "raises on invalid arg count" do
          expect { adder_instance.invoke("add", 1) }
            .to raise_error(ArgumentError, /(given 1, expected 2)/)
        end

        it "raises on invalid arg type" do
          expect { adder_instance.invoke("add", nil, nil) }
            .to raise_error(TypeError, "no implicit conversion of nil into Integer (param at index 0)")
        end
      end
    end
  end
end
