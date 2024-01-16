require "wasmtime"

engine = Wasmtime::Engine.new
mod = Wasmtime::Module.from_file(engine, "spec/fixtures/wasi-debug.wasm")

linker = Wasmtime::Linker.new(engine, wasi: true)

wasi_ctx = Wasmtime::WasiCtxBuilder.new
  .set_stdin_string("hi!")
  .inherit_stdout
  .inherit_stderr
  .set_argv(ARGV)
  .set_env(ENV)
  .build
store = Wasmtime::Store.new(engine, wasi_ctx: wasi_ctx)

instance = linker.instantiate(store, mod)
instance.invoke("_start")
