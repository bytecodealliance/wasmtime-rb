require "spec_helper"

module Wasmtime
  RSpec.describe Extern do
    cases = {
      f: [:to_func, Func],
      m: [:to_memory, Memory],
      t: [:to_table, Table],
      g: [:to_global, Global]
    }

    cases.each do |name, (meth, klass)|
      describe "##{meth}" do
        it "returns an instance of #{klass}" do
          extern_mod = new_extern_module
          export = extern_mod.exports[name.to_s]

          expect(export.public_send(meth)).to be_instance_of(klass)
        end

        it "raises an error when extern is not a #{klass}" do
          extern_mod = new_extern_module
          invalid_methods = cases.flat_map { |_, (meth, _)| meth } - [meth]

          invalid_methods.each do |invalid_method|
            export = extern_mod.exports[name.to_s]
            expect { export.public_send(invalid_method) }.to raise_error(Wasmtime::ConversionError)
          end
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
          (table (export "t") 1 funcref)
          (global (export "g") (mut i32) (i32.const 1))
        )
      WAT
    end
  end
end
