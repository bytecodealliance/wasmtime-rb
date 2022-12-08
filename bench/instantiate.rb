require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Linker.new(engine)
  mod = Wasmtime::Module.new(engine, "(module)")

  x.report("Linker#instantiate") do
    store = Wasmtime::Store.new(engine)
    linker.instantiate(store, mod)
  end

  x.report("Instance#new") do
    store = Wasmtime::Store.new(engine)
    Wasmtime::Instance.new(store, mod)
  end

  x.compare!
end
