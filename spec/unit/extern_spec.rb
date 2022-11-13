require "spec_helper"

module Wasmtime
  RSpec.describe Extern do
    cases = {
      f: [:to_func, Func],
      m: [:to_memory, Memory]
    }

    describe "#ty" do
      cases.each do |name, (meth, klass)|
        it "exposes the #{name.inspect} export with #{klass.inspect} type" do
          extern_mod = new_extern_module
          expect(extern_mod.exports[name.to_s].public_send(meth)).to be_instance_of(klass)
        end
      end
    end

    def new_extern_module
      compile <<~WAT
        (module
          (func (export "f"))
          (memory (export "m") 1)
        )
      WAT
    end
  end
end
