require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/hello.wat")
store = Wasmtime::Store.new(engine, {count: 0})
func = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], [])) do |caller|
  puts "Hello from Func!"
  caller.store_data[:count] += 1
end

instance = Wasmtime::Instance.new(store, mod, [func])
instance.invoke("run")

puts "Store's count: #{store.data[:count]}"
