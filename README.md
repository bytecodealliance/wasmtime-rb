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

Add the `wasmtime` gem to your Gemfile and run `bundle install`:

```ruby
gem "wasmtime"
```

Alternatively, you can install the gem manually:

```sh
gem install wasmtime
```

### Precompiled gems

We recommend installing the `wasmtime` precompiled gems available for Linux, macOS, and Windows. Installing a precompiled gem avoids the need to compile from source code, which is generally slower and less reliable.

When installing the `wasmtime` gem for the first time using `bundle install`, Bundler will automatically download the precompiled gem for your current platform. However, you will need to inform Bundler of any additional platforms you plan to use.

To do this, lock your Bundle to the required platforms you will need from the list of supported platforms below:

```sh
bundle lock --add-platform x86_64-linux # Standard Linux (e.g. Heroku, GitHub Actions, etc.)
bundle lock --add-platform x86_64-linux-musl # MUSL Linux deployments (i.e. Alpine Linux)
bundle lock --add-platform aarch64-linux # ARM64 Linux deployments (i.e. AWS Graviton2)
bundle lock --add-platform x86_64-darwin # Intel MacOS (i.e. pre-M1)
bundle lock --add-platform arm64-darwin # Apple Silicon MacOS  (i.e. M1)
```

## Usage

Example usage:

```ruby
require "wasmtime"

# Create an engine. Generally, you only need a single engine and can
# re-use it throughout your program.
engine = Wasmtime::Engine.new

# Compile a Wasm module from either Wasm or WAT. The compiled module is
# specific to the Engine's configuration.
mod = Wasmtime::Module.new(engine, <<~WAT)
  (module
    (func $hello (import "" "hello"))
    (func (export "run") (call $hello))
  )
WAT

# Create a store. Store can keep state to be re-used in Funcs.
store = Wasmtime::Store.new(engine, {count: 0})

# Define a Wasm function from Ruby code.
func = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], [])) do |caller|
  puts "Hello from Func!"
  caller.store_data[:count] += 1
  puts "Ran #{caller.store_data[:count]} time(s)"
end

# Build the Wasm instance by providing its imports.
instance = Wasmtime::Instance.new(store, mod, [func])

# Run the `run` export.
instance.invoke("run")

# Or: get the `run` export and call it.
instance.export("run").to_func.call
```

For more, see [examples](https://github.com/bytecodealliance/wasmtime-rb/tree/main/examples)
or the [API documentation](https://bytecodealliance.github.io/wasmtime-rb/latest/).
