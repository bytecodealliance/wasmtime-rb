Example WASI program used to test the WASI integration.

To update:

```shell
cargo build --release && \
    wasm-opt -O \
    --enable-bulk-memory \
    target/wasm32-wasip1/release/wasi-debug.wasm \
    -o ../wasi-debug.wasm && \
cargo build --target=wasm32-wasip2 --release && \
    cp target/wasm32-wasip2/release/wasi-debug.wasm \
    ../wasi-debug-p2.wasm
```
