# Poll HTTP Server Example

### Compile & Run

```
$ cargo build --target wasm32-wasi
$ wasmedge target/wasm32-wasi/debug/poll_http_server.wasm
```

### Test

```
$ curl -X POST --data "Hello" "http://127.0.0.1:1234/"
```

