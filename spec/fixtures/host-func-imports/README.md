Wasm component fixture to test host function imports via `func_new`.

This component imports various host functions with different type signatures
and provides guest functions that call them.

Prerequisite: `cargo install cargo-component`

To rebuild, run the following from the wasmtime-rb's root:
```
(
  cd spec/fixtures/host-func-imports && \
  cargo component build --release  && \
  cp target/wasm32-unknown-unknown/release/host_func_imports.wasm ../
)
```
