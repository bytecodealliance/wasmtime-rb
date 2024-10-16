require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Component::Linker.new(engine)
  component = Wasmtime::Component::Component.from_file(engine, "spec/fixtures/component_types.wasm")
  store = Wasmtime::Store.new(engine)
  instance = linker.instantiate(store, component)

  point_record = {"x" => 1, "y" => 2}

  x.report("identity point record") do
    instance.invoke("id-record", point_record)
  end

  x.report("identity u32") do
    instance.invoke("id-u32", 10)
  end
end
