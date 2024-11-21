require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  linker = Wasmtime::Component::Linker.new(engine)
  component = Wasmtime::Component::Component.from_file(engine, "spec/fixtures/component_types.wasm")
  store = Wasmtime::Store.new(engine)
  instance = linker.instantiate(store, component)
  id_large_record = instance.get_func("id-large-record")
  large_record = (1..20).each_with_object({}) { |i, hash| hash[:"field#{i}"] = i }

  x.report("identity large record (symbols)") do
    record = id_large_record.call(large_record)
    # do something to also incur the cost of accessing the hash
    record.keys.each { |k| record[k] }
  end
end
