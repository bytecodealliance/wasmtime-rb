# Development Notes

## Goal

This library aims to be the official Ruby bindings for Wasmtime.

Looking at the official Wasmtime bindings, they use the C API and their API follows the one provided by the C API. For consistency, ease of deployment and making it easy to follow Wasmtime's releases, this gem will follow a similar approach.

## How to build this?

Outside of the Rust options, I see 3 ways we can build this:
- [libffi-ruby](https://github.com/ffi/ffi)
  - Maybe using codegen with [ffi_gen](https://github.com/ffi/ffi_gen)?
- [Fiddle](https://github.com/ruby/fiddle)
- plain c ext

No idea which is best, nor what are the tradeoffs. For now, I'll get with either of the FFI libs because it looks easier, and take it from there.

## How to bundle shared library and C api?

To avoid the dependency on the Rust compiler, this gem will use the pre-built shared libraries from Wasmtime.

How should the shared library be distributed? Options:
1. Downloaded at gem install or build time. Pro: smaller footprint because we know which lib to download for the (os, arch) tuple.
1. Bundled with the gem. This would require bundling all libraries.

(1) looks like the way to go, but unsure on the challenges yet.

## Links

Useful links to docs or other projects for inspiration.

- [Nokogiri's usage of mini_portile](https://github.com/sparklemotion/nokogiri/blob/0e75392d49ca3758dcdfc6610c9c22ccdc23001c/ext/nokogiri/extconf.rb#L418)
- Wasmtime official libraries: [go](), [.net]().
- Wasmtime C API: [doc](https://docs.wasmtime.dev/c-api/), [examples](https://github.com/bytecodealliance/wasmtime/tree/main/examples)
- [Mike Dalession's Ruby C Extensions, Explained](https://github.com/flavorjones/ruby-c-extensions-explained)
 - [mini_portile2](https://github.com/flavorjones/mini_portile) for downloading the Wasmtime library, I think?


