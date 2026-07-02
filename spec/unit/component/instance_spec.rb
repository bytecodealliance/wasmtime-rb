require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Instance do
      before(:all) do
        @adder_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_adder.wat")
        @types_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_types.wasm")
      end

      let(:linker) { Linker.new(engine) }
      let(:adder_instance) { linker.instantiate(store, @adder_component) }
      let(:types_instance) { linker.instantiate(store, @types_component) }

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

        it "accepts a previously resolved ExportIndex" do
          index = adder_instance.get_export_index(["adder", "add"])
          expect(adder_instance.get_func(index)).to be_instance_of(Wasmtime::Component::Func)
        end
      end

      describe "#get_export_index" do
        it "returns an ExportIndex for a root export" do
          expect(adder_instance.get_export_index("add")).to be_instance_of(Wasmtime::Component::ExportIndex)
        end

        it "returns an ExportIndex for a nested export" do
          expect(types_instance.get_export_index(["resource", "wrapped-string"]))
            .to be_instance_of(Wasmtime::Component::ExportIndex)
        end

        it "returns nil for a missing export" do
          expect(adder_instance.get_export_index("no")).to be_nil
          expect(adder_instance.get_export_index(["add", "no"])).to be_nil
        end

        it "raises for invalid arg" do
          expect { adder_instance.get_export_index(3) }
            .to raise_error(TypeError, /invalid argument for component index/)

          expect { adder_instance.get_export_index([nil]) }
            .to raise_error(TypeError, /invalid argument for component index/)
        end

        it "allows chaining a resolved ExportIndex into further lookups" do
          resource_index = types_instance.get_export_index("resource")

          expect(types_instance.get_export_index([resource_index, "wrapped-string"]))
            .to eq(types_instance.get_export_index(["resource", "wrapped-string"]))

          expect(types_instance.get_func([resource_index, "[constructor]wrapped-string"]))
            .to be_instance_of(Wasmtime::Component::Func)
        end
      end

      describe "#get_resource" do
        it "returns a ResourceType for an exported resource" do
          expect(types_instance.get_resource(["resource", "wrapped-string"]))
            .to be_instance_of(Wasmtime::Component::ResourceType)
        end

        it "returns nil for a non-resource or missing export" do
          expect(types_instance.get_resource(["resource", "resource-owned"])).to be_nil
          expect(types_instance.get_resource(["resource", "no"])).to be_nil
          expect(types_instance.get_resource("no")).to be_nil
        end

        it "raises for invalid arg" do
          expect { types_instance.get_resource(3) }
            .to raise_error(TypeError, /invalid argument for component index/)
        end

        it "returns equal ResourceTypes for repeated lookups of the same resource" do
          a = types_instance.get_resource(["resource", "wrapped-string"])
          b = types_instance.get_resource(["resource", "wrapped-string"])

          expect(a).to eq(b)
        end
      end
    end
  end
end
