require "spec_helper"

module Wasmtime
  RSpec.describe Instance do
    it "exposes the exports" do
      instance = compile <<~WAT
        (module
          (func $module/hello (result i32)
            i32.const 1
          )
          (export "hello" (func $module/hello))
        )
      WAT

      exports = instance.exports
      type_names = exports.transform_values(&:type_name)

      expect(exports).to include(hello: be_a(Export))
      expect(type_names).to eq(hello: :func)
    end
  end
end
