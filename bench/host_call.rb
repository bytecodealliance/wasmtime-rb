require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  [4, 16, 64, 128, 256].each do |n|
    result_type_wat = Array.new(n) { |_| :i32 }.join(" ")
    mod = Wasmtime::Module.new(engine, <<~WAT)
      (module
        (import "host" "succ" (func (param i32) (result #{result_type_wat})))
        (export "run" (func 0)))
    WAT
    linker = Wasmtime::Linker.new(engine)
    results = Array.new(n) { |_| :i32 }
    result_array = Array.new(n) { |i| i }
    linker.func_new("host", "succ", [:i32], results) do |_caller, arg1|
      result_array
    end

    x.report("Call host func (#{n} args)") do
      store = Wasmtime::Store.new(engine)
      linker.instantiate(store, mod).invoke("run", 101)
    end
  end

  x.compare!
end
