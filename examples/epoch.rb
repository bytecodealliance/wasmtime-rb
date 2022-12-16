require "wasmtime"

config = Wasmtime::Config.new
config.epoch_interruption = true
engine = Wasmtime::Engine.new(config)
# Re-use fibonacci function from the Fuel example
mod = Wasmtime::Module.from_file(engine, "examples/fuel.wat")
store = Wasmtime::Store.new(engine)
instance = Wasmtime::Instance.new(store, mod)

# Store starts with an epoch deadline of 0, meaning Wasm execution
# will be halted right away.
puts "Running Wasm with default epoch deadline of 0..."
begin
  instance.invoke("fibonacci", 5)
  raise "Unexpected: Wasm executed past deadline"
rescue Wasmtime::Trap => trap
  puts "  Wasm trap, code: #{trap.code.inspect}"
  puts
end

# Epoch deadline is manipulated with `Store#set_epoch_deadline`.
store.set_epoch_deadline(1)
puts "Running Wasm with default epoch deadline of 1..."
puts "  result: #{instance.invoke("fibonacci", 5)}"
puts

# The engine's epoch can be incremented manually with `Engine#increment_epoch`.
engine.increment_epoch
puts "Running Wasm after incrementing epoch past the store's deadline..."
begin
  instance.invoke("fibonacci", 5)
  raise "Unexpected: Wasm executed past deadline"
rescue Wasmtime::Trap => trap
  puts "  Wasm trap, code: #{trap.code.inspect}"
  puts
end

# The engine provides a method to increment epoch based on a timer.
# This is done with native thread because Wasm execution does not release
# Ruby's Global VM lock.
puts "Setting the store's deadline to be 2 ticks from current epoch..."
store.set_epoch_deadline(2)

puts "Incrementing epoch interval every 100ms..."
engine.start_epoch_interval(100)
start_time = Process.clock_gettime(Process::CLOCK_MONOTONIC)
puts "Computing fibonacci of 100..."
begin
  instance.invoke("fibonacci", 100)
  raise "Unexpected: computed fibonacci of 100 in 200ms"
rescue Wasmtime::Trap => _
  elapsed_ms = (Process.clock_gettime(Process::CLOCK_MONOTONIC) - start_time) * 1000
  # This should be around 200ms
  puts "  Wasm trapped after #{elapsed_ms.round}ms"
end
