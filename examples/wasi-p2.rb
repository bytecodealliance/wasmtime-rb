require "wasmtime"

engine = Wasmtime::Engine.new
component = Wasmtime::Component::Component.from_file(engine, "spec/fixtures/wasi-debug-p2.wasm")

linker = Wasmtime::Component::Linker.new(engine)
Wasmtime::WASI::P2.add_to_linker_sync(linker)

wasi_config = Wasmtime::WasiConfig.new
  .set_stdin_string("hi!")
  .inherit_stdout
  .inherit_stderr
  .set_argv(ARGV)
  .set_env(ENV)
store = Wasmtime::Store.new(engine, wasi_config: wasi_config)

Wasmtime::Component::WasiCommand.new(store, component, linker).call_run(store)
