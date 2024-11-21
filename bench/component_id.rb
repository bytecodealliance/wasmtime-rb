require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Component::Linker.new(engine)
  component = Wasmtime::Component::Component.from_file(engine, "spec/fixtures/component_types.wasm")
  store = Wasmtime::Store.new(engine)
  instance = linker.instantiate(store, component)
  id_record = instance.get_func("id-record")
  id_u32 = instance.get_func("id-u32")

  point_record = {"x" => 1, "y" => 2}

  x.report("identity point record") do
    id_record.call(point_record)
  end

  x.report("identity u32") do
    id_u32.call(10)
  end
end
