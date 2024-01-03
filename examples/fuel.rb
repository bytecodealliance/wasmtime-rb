require "wasmtime"

engine = Wasmtime::Engine.new(consume_fuel: true)
mod = Wasmtime::Module.from_file(engine, "examples/fuel.wat")

store = Wasmtime::Store.new(engine)
store.set_fuel(10_000)

instance = Wasmtime::Instance.new(store, mod)

begin
  (1..).each do |i|
    initial_fuel = store.get_fuel
    result = instance.invoke("fibonacci", i)
    fuel_consumed = initial_fuel - store.get_fuel
    puts "fib(#{i}) = #{result} [consumed #{fuel_consumed} fuel]"
  end
rescue Wasmtime::Trap => trap
  puts
  puts "Wasm trap, code: #{trap.code.inspect}"
end
