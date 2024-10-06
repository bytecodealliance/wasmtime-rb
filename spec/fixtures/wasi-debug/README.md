Example WASI program used to test the WASI integration.

To update:

```shell
cargo build --release && \
    wasm-opt -O \
    --enable-bulk-memory \
    target/wasm32-wasip1/release/wasi-debug.wasm \
    -o ../wasi-debug.wasm
```
