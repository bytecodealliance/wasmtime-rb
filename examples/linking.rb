require "wasmtime"

engine = Wasmtime::Engine.new

# Create a linker to link modules together.
linker = Wasmtime::Linker.new(engine)
# We want to use WASI with # the linker, so we call add_to_linker_sync.
Wasmtime::WASI::P1.add_to_linker_sync(linker)

mod1 = Wasmtime::Module.from_file(engine, "examples/linking1.wat")
mod2 = Wasmtime::Module.from_file(engine, "examples/linking2.wat")

wasi_config = Wasmtime::WasiConfig.new
  .inherit_stdin
  .inherit_stdout

store = Wasmtime::Store.new(engine, wasi_p1_config: wasi_config)

# Instantiate `mod2` which only uses WASI, then register
# that instance with the linker so `mod1` can use it.
instance2 = linker.instantiate(store, mod2)
linker.instance(store, "linking2", instance2)

# Perform the final link and execute mod1's "run" function.
instance1 = linker.instantiate(store, mod1)
instance1.invoke("run")
