Example WASI program used to test the WASI deterministic context integration.

To update:

```shell
cargo build --release && \
    wasm-opt -O \
    --enable-bulk-memory \
    target/wasm32-wasip1/release/wasi-deterministic.wasm \
    -o ../wasi-deterministic.wasm && \
cargo build --target=wasm32-wasip2 --release && \
    cp target/wasm32-wasip2/release/wasi-deterministic.wasm \
    ../wasi-deterministic-p2.wasm
```

