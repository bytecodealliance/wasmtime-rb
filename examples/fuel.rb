require "wasmtime"

config = Wasmtime::Config.new
config.consume_fuel = true
engine = Wasmtime::Engine.new(config)
mod = Wasmtime::Module.from_file(engine, "examples/fuel.wat")

store = Wasmtime::Store.new(engine)
store.add_fuel(10_000)

instance = Wasmtime::Instance.new(store, mod)

begin
  (1..).each do |i|
    fuel_before = store.fuel_consumed
    result = instance.invoke("fibonacci", i)
    fuel_consumed = store.fuel_consumed - fuel_before
    puts "fib(#{i}) = #{result} [consumed #{fuel_consumed} fuel]"
  end
rescue Wasmtime::Trap => trap
  puts
  puts "Wasm trap, code: #{trap.code.inspect}"
end
