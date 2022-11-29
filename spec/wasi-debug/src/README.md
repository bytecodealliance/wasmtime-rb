Example WASI program used to test the WASI integration. To update:

```shell
cargo wasi build --release
cp target/wasm32-wasi/release/wasi-debug.wasm ../fixtures/wasi-debug.wasm
```
