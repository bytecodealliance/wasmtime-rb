require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "spec/fixtures/wasi-debug.wasm")

linker = Wasmtime::Linker.new(engine, wasi: true)

wasi_config = Wasmtime::WasiConfig.new
  .set_stdin_string("hi!")
  .inherit_stdout
  .inherit_stderr
  .set_argv(ARGV)
  .set_env(ENV)
store = Wasmtime::Store.new(engine, wasi_config: wasi_config)

instance = linker.instantiate(store, mod)
instance.invoke("_start")
