Wasm component fixture to test converting types back and forth between the guest
and the Ruby host.

Prerequisite: `cargo install cargo-component`

To rebuild, run the following from the wasmtime-rb's root:
```
(
  cd spec/fixtures/component-types && \
  cargo component build --release  && \
  cp target/wasm32-unknown-unknown/release/component_types.wasm ../
)
```
