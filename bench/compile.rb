require_relative "bench"

Bench.ips do |x|
  engine = Wasmtime::Engine.new

  x.report("empty module") do
    Wasmtime::Module.new(engine, "(module)")
  end
end
