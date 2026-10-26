require_relative "bench"

engine = Wasmtime::Engine.new
store = Wasmtime::Store.new(engine)
mod = Wasmtime::Module.new(engine, <<~WAT)
  (module
    (func (export "identity") (param externref) (result externref)
      (local.get 0)))
WAT
identity = Wasmtime::Instance.new(store, mod).export("identity").to_func
value_class = Class.new
value = value_class.new

Bench.ips do |x|
  x.report("externref roundtrip (1000 calls + Wasmtime GC)") do
    1000.times { identity.call(value) }
    store.gc
  end

  x.report("externref churn (1000 objects + Wasmtime GC)") do
    1000.times { identity.call(value_class.new) }
    store.gc
  end

  x.compare!
end

store.gc
GC.start(full_mark: true, immediate_sweep: true)
puts "Ruby values still alive after collection: #{ObjectSpace.each_object(value_class).count} (expected 1)"
