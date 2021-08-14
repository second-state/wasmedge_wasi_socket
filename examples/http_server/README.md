# WasmEdge WASI Socket Http Client Demo
a echo http server

#install
```shell
cargo install cargo-wasi
```

#build
```shell
cargo wasi build --release
```

#run
wasmedge(0.8.2)
```shell
wasmedge http_server.wasm
```