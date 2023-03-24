module Wasmtime
  RSpec.describe "Crash" do
    # see https://github.com/bytecodealliance/wasmtime-rb/issues/156
    it "ensures params are never GC'd" do
      store = Store.new(Wasmtime::Engine.new, Object.new)
      func = Func.new(store, [], [:i32, :i32]) { [1, nil] }

      1_000.times do
        func.call
      rescue Wasmtime::ResultError
      end
    end
  end
end
