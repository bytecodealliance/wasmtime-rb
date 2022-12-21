require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Linker.new(engine)
  mod = Wasmtime::Module.from_file(engine, "examples/gcd.wat")
  store = Wasmtime::Store.new(engine)
  instance = linker.instantiate(store, mod)
  func = instance.export("gcd").to_func

  x.report("Instance#invoke") do
    instance.invoke("gcd", 5, 1)
  end

  x.report("Func#call") do
    func.call(5, 1)
  end

  x.compare!
end
