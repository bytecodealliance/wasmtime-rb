require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/memory.wat")
store = Wasmtime::Store.new(engine)
instance = Wasmtime::Instance.new(store, mod)
memory = instance.export("memory").to_memory
size_fn = instance.export("size").to_func
load_fn = instance.export("load").to_func
store_fn = instance.export("store").to_func

puts "Checking memory..."
puts "  size: #{memory.size}" # => 2
puts "  read(0, 1): #{memory.read(0, 1).inspect}" # => "\x00"
puts
puts "  size_fn.call: #{size_fn.call}" # => 2
puts "  load_fn.call(0): #{load_fn.call(0)}" # => 0
puts

puts "Reading out of bounds..."
begin
  load_fn.call(0x20000)
rescue Wasmtime::Trap => error
  puts "  Trap! code: #{error.code.inspect}"
end
puts

puts "Mutating memory..."
memory.write(0x1002, "\x06")
puts "  load_fn.call(0x1002): #{load_fn.call(0x1002).inspect}" # => 6
store_fn.call(0x1003, 7)
puts "  load_fn.call(0x1003): #{load_fn.call(0x1003).inspect}" # => 7
puts

puts "Growing memory..."
memory.grow(1)
puts "  new size: #{memory.size}" # => 3
puts

puts "Creating stand-alone memory..."
memory = Wasmtime::Memory.new(store, min_size: 5, max_size: 5) # size in pages
puts "  size: #{memory.size}" # => 5
puts
puts "Growing beyond limit fails..."
begin
  memory.grow(1)
rescue Wasmtime::Error => error
  puts "  exception: #{error.inspect}"
end
