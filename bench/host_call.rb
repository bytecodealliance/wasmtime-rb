require_relative "bench"

# Call host func (4 args)    337.181k (± 3.7%) i/s -      1.692M in   5.024584s
# Call host func (16 args)   296.615k (± 5.2%) i/s -      1.498M in   5.064241s
# Call host func (64 args)   217.487k (± 3.2%) i/s -      1.106M in   5.090547s
# Call host func (128 args)  119.689k (± 3.8%) i/s -    605.136k in   5.063428s

# Call host func (4 args):   333800.6 i/s
# Call host func (16 args):  291889.7 i/s - 1.14x  slower
# Call host func (64 args):  185375.6 i/s - 1.80x  slower
# Call host func (128 args): 97043.2 i/s - 3.44x  slower

Bench.ips do |x|
  engine = Wasmtime::Engine.new
  [4, 16, 64, 128, 256].each do |n|
    result_type_wat = Array.new(n) { |_| :i32 }.join(" ")
    mod = Wasmtime::Module.new(engine, <<~WAT)
      (module
        (import "host" "succ" (func (param i32) (result #{result_type_wat})))
        (export "run" (func 0)))
    WAT
    linker = Wasmtime::Linker.new(engine)
    results = Array.new(n) { |_| :i32 }
    result_array = Array.new(n) { |i| i }
    linker.func_new("host", "succ", [:i32], results) do |_caller, arg1|
      result_array
    end

    x.report("Call host func (#{n} args)") do
      store = Wasmtime::Store.new(engine)
      linker.instantiate(store, mod).invoke("run", 101)
    end
  end

  x.compare!
end
