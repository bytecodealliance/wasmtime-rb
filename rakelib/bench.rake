require "benchmark/ips"
require "wasmtime"

namespace :bench do
  task "compile" do
    engine = Wasmtime::Engine.new
    Benchmark.ips do |x|
      x.report("empty module") do
        Wasmtime::Module.new(engine, "(module)")
      end
    end
  end

  task "instantiate" do
    engine = Wasmtime::Engine.new
    linker = Wasmtime::Linker.new(engine)
    mod = Wasmtime::Module.new(engine, "(module)")

    Benchmark.ips do |x|
      x.report("Linker#instantiate") do
        store = Wasmtime::Store.new(engine)
        linker.instantiate(store, mod)
      end
      x.report("Instance#new") do
        store = Wasmtime::Store.new(engine)
        Wasmtime::Instance.new(store, mod)
      end
    end
  end

  task "func_call" do
    engine = Wasmtime::Engine.new
    linker = Wasmtime::Linker.new(engine)
    mod = Wasmtime::Module.from_file(engine, "examples/gcd.wat")

    Benchmark.ips do |x|
      x.report("Instance#invoke") do
        store = Wasmtime::Store.new(engine)
        linker.instantiate(store, mod).invoke("gcd", 5, 1)
      end
      x.report("Func#call") do
        store = Wasmtime::Store.new(engine)
        linker.instantiate(store, mod).export("gcd").to_func.call(5, 1)
      end
    end
  end

  task "host_call" do
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

    Benchmark.ips do |x|
      x.report("Call host func") do
        store = Wasmtime::Store.new(engine)
        linker.instantiate(store, mod).invoke("run", 101)
      end
    end
  end
end
