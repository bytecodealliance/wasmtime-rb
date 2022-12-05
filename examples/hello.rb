require "wasmtime"

class MyData
  attr_reader :count

  def initialize
    @count = 0
  end

  def increment!
    @count += 1
  end
end

data = MyData.new
engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "examples/hello.wat")
store = Wasmtime::Store.new(engine, data)
func = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], [])) do |caller|
  puts "Hello from Func!"
  caller.store_data.increment!
end

instance = Wasmtime::Instance.new(store, mod, [func])
instance.invoke("run")

puts "Store's count: #{store.data.count}"
