require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/hello.wat")
data = {count: 0}
store = Wasmtime::Store.new(engine, data)
func = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], [])) do |caller|
  puts "Hello from Func!"
  caller.store_data[:count] += 1
end

instance = Wasmtime::Instance.new(store, mod, [func])
instance.invoke("run")

puts "Store's count: #{store.data[:count]}"
