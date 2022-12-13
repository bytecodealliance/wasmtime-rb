require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/externref.wat")
store = Wasmtime::Store.new(engine)
instance = Wasmtime::Instance.new(store, mod)

# externrefs can be any Ruby object, here we're using a String.
hello_world = +"Hello World"

puts "Set and get from table..."
table = instance.export("table").to_table
table.set(3, hello_world)
puts "  get: #{table.get(3).inspect}" # => Hello World
puts "  same Ruby object: #{table.get(3).eql?(hello_world).inspect}" # => true
puts

puts "Set and get from global..."
global = instance.export("global").to_global
global.set(hello_world)
puts "  get: #{global.get.inspect}" # => "Hello World"
puts

puts "Return an externref from a function..."
func = instance.export("func").to_func
puts "  return: #{func.call(hello_world).inspect}" # => "Hello World"
puts

puts "nil is a valid externref..."
puts "  nil roundtrip: #{func.call(nil).inspect}" # => nil
puts

puts "nil is also a 'null reference' externref..."
puts "  null externref: #{table.get(6).inspect}" # => nil
