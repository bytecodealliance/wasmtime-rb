<div align="center">
  <h1><code>wasmtime-rb</code></h1>

  <p>
    <strong>Ruby embedding of
    <a href="https://github.com/bytecodealliance/wasmtime">Wasmtime</a></strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/wasmtime-rb/actions?query=workflow%3ACI">
      <img src="https://github.com/bytecodealliance/wasmtime-rb/workflows/CI/badge.svg" alt="CI status"/>
    </a>
  </p>
</div>

## Status

The Wasmtime Ruby bindings are still under development, [some features](https://github.com/bytecodealliance/wasmtime-rb/issues?q=is%3Aissue+is%3Aopen+label%3A%22missing+feature%22) are still missing.

## Installation

Install from RubyGems

```shell
gem install wasmtime
```

Or use in your Gemfile:

```ruby
gem "wasmtime", "~> 0.3.0"
```

## Usage

Example usage:

```ruby
require "wasmtime"

# Create an engine. Generally, you only need a single engine and can
# re-use it a throughout your program.
engine = Wasmtime::Engine.new

# Compile a Wasm module from either Wasm or WAT. The compiled module is
# specific to the Engine's configuration.
mod = Wasmtime::Module.new(engine, <<~WAT
  (module
    (func $hello (import "" "hello"))
    (func (export "run") (call $hello))
  )
WAT

# Create a store. Store can keep state to be re-used in Funcs.
store = Wasmtime::Store.new(engine, { count: 0 })

# Define a Wasm function from Ruby code.
func = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], [])) do |caller|
  puts "Hello from Func!"
  puts "Ran #{caller[:count]} time(s)"
end

# Build the Wasm instance by providing its imports
instance = Wasmtime::Instance.new(store, mod, [func])

# Run the `run` export.
instance.invoke("run")

# Or: get the `run` export and call it
instance.export("run").call
```

For more, see [examples](https://github.com/bytecodealliance/wasmtime-rb/tree/main/examples)
or the [API documentation](https://bytecodealliance.github.io/wasmtime-rb/latest/).
