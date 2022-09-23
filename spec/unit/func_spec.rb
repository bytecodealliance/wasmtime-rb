require "spec_helper"

module Wasmtime
  RSpec.describe Func do
    let(:engine) { Engine.new }
    let(:store) { Store.new(engine, {}) }

    describe(".new") do
      it("accepts a proc") do
        runs = 0
        func = Func.new(store, "TODO functype", true, -> { runs += 1 })
        mod = Wasmtime::Module.new(engine, <<~WAT)
          (module
            (import "a" "b" (func))
            (start 0))
        WAT
        Wasmtime::Instance.new(store, mod, [func])
        expect(runs).to eq(1)
        Wasmtime::Instance.new(store, mod, [func])
        expect(runs).to eq(2)
      end
    end
  end
end
