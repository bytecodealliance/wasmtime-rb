Example WASI program used to test WASI preopened directories

To update:

```shell
cargo build --release && \
    wasm-opt -O \
    --enable-bulk-memory \
    target/wasm32-wasip1/release/wasi-fs.wasm \
    -o ../wasi-fs.wasm && \
cargo build --target=wasm32-wasip2 --release && \
    cp target/wasm32-wasip2/release/wasi-fs.wasm \
    ../wasi-fs-p2.wasm
```
