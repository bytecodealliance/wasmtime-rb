require "wasmtime"

engine = Wasmtime::Engine.new

# Create a linker to link modules together. We want to use WASI with
# the linker, so we pass in `wasi: true`.
linker = Wasmtime::Linker.new(engine, wasi: true)

mod1 = Wasmtime::Module.from_file(engine, "examples/linking1.wat")
mod2 = Wasmtime::Module.from_file(engine, "examples/linking2.wat")

wasi_ctx_builder = Wasmtime::WasiCtxBuilder.new
  .inherit_stdin
  .inherit_stdout

store = Wasmtime::Store.new(engine, wasi_ctx: wasi_ctx_builder)

# Instantiate `mod2` which only uses WASI, then register
# that instance with the linker so `mod1` can use it.
instance2 = linker.instantiate(store, mod2)
linker.instance(store, "linking2", instance2)

# Perform the final link and execute mod1's "run" function.
instance1 = linker.instantiate(store, mod1)
instance1.invoke("run")
