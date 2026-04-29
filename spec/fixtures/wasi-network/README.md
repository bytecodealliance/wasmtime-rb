Example WASI program used to test the network integration.

Tests TCP connections, UDP sockets, and DNS resolution. This is only supported
using WASI preview 2.

To update:

```shell
cargo build --target=wasm32-wasip2 --release && \
    cp target/wasm32-wasip2/release/wasi-network.wasm \
    ../wasi-network-p2.wasm
```
