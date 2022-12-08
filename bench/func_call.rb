require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Linker.new(engine)
  mod = Wasmtime::Module.from_file(engine, "examples/gcd.wat")

  x.report("Instance#invoke") do
    store = Wasmtime::Store.new(engine)
    linker.instantiate(store, mod).invoke("gcd", 5, 1)
  end

  x.report("Func#call") do
    store = Wasmtime::Store.new(engine)
    linker.instantiate(store, mod).export("gcd").to_func.call(5, 1)
  end

  x.compare!
end
