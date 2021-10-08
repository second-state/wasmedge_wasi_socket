# WasmEdge WASI Socket Http Server Demo

This demo runs an echo server on `localhost`.

## Build

```shell
cargo build --target wasm32-wasi --release
```

## Run

```shell
wasmedge target/wasm32-wasi/release/http_server.wasm
```
