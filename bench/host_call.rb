require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  mod = Wasmtime::Module.new(engine, <<~WAT)
    (module
      (import "host" "succ" (func (param i32) (result i32)))
      (export "run" (func 0)))
  WAT
  linker = Wasmtime::Linker.new(engine)
  linker.func_new("host", "succ", Wasmtime::FuncType.new([:i32], [:i32])) do |_caller, arg1|
    arg1.succ
  end

  x.report("Call host func") do
    store = Wasmtime::Store.new(engine)
    linker.instantiate(store, mod).invoke("run", 101)
  end

  x.compare!
end
