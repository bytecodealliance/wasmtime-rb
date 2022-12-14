require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/multi.wat")
store = Wasmtime::Store.new(engine)

# Host call with multiple params and results
callback = Wasmtime::Func.new(store, [:i32, :i64], [:i64, :i32]) do |_caller, a, b|
  # Return an array with 2 elements for 2 results
  [b + 1, a + 1]
end

instance = Wasmtime::Instance.new(store, mod, [callback])

g = instance.export("g").to_func

puts "Calling export 'g'..."
result = g.call(1, 3) # => [2, 4]
puts "  g result: #{result.inspect}"
puts

round_trip_many = instance.export("round_trip_many").to_func

puts "Calling export 'round_trip_many'..."
result = round_trip_many.call(0, 1, 2, 3, 4, 5, 6, 7, 8, 9) # => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
puts "  round_trip_many result: #{result.inspect}"
