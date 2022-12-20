require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/gcd.wat")
store = Wasmtime::Store.new(engine)
instance = Wasmtime::Instance.new(store, mod)

puts "gcd(6, 27) = #{instance.invoke("gcd", 6, 27)}"
