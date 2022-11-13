require "spec_helper"

module Wasmtime
  RSpec.describe Extern do
    cases = {
      f: [:to_func, Func],
      m: [:to_memory, Memory]
    }

    cases.each do |name, (meth, klass)|
      describe "##{meth}" do
        it "returns an instance of #{klass}" do
          extern_mod = new_extern_module

          expect(extern_mod.exports[name.to_s].public_send(meth)).to be_instance_of(klass)
        end
      end
    end

    describe "#inspect" do
      it "looks pretty" do
        result = new_extern_module.exports["f"].inspect

        expect(result).to match(/\A#<Wasmtime::Extern:0x\h{16} @value=#<Wasmtime::Func:0x\h{16}>>$/)
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
