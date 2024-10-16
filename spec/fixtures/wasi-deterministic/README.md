Example WASI program used to test the WASI deterministic context integration.

To update:

```shell
cargo build --release && \
    wasm-opt -O \
    --enable-bulk-memory \
    target/wasm32-wasip1/release/wasi-deterministic.wasm \
    -o ../wasi-deterministic.wasm
```

