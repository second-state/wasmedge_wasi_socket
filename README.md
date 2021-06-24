# WasmEdge WASI Socket

This is an example of how to run socket program in wasmedge with a wasm compiled from rust.

Clone [WasmEdge](https://github.com/WasmEdge/WasmEdge) and follow the build step to build wasmedge.

# TCP Stream Example with WasmEdge

This is a example of using wasmedge as a socket client.

```
cargo build --example tcp_stream --target wasm32-unknown-unknown
```

Set up a server on your localhost with [ncat](https://nmap.org/ncat).

```
ncat -kvlp 1234
```

Copy wasm into wasmedge directory and run it. Wasmedge would send message "hello" to a server at `localhost:1234`.

```
cp <path-to-w13e_wasi_socket>/target/wasm32-unknown-unknown/debug/examples/tcp_stream.wasm <path-to-wasmedge>
./wasmedge --reactor ./tcp_stream.wasm main 0 0
```

The server should get the message "hello".
```
$ ncat -kvlp 1234 
Ncat: Version 7.91 ( https://nmap.org/ncat )
Ncat: Listening on :::1234
Ncat: Listening on 0.0.0.0:1234
Ncat: Connection from 127.0.0.1.
Ncat: Connection from 127.0.0.1:56366.
hello
```

# TCP Listener Example with WasmEdge

This is a example of using wasmedge as a socket server.

```
cargo build --example tcp_listener --target wasm32-unknown-unknown
```

Copy wasm into wasmedge directory and run it. This should setup a tcp listener at `localhost:1234` in wasmedge.

```
cp <path-to-w13e_wasi_socket>/target/wasm32-unknown-unknown/debug/examples/tcp_listener.wasm <path-to-wasmedge>
./wasmedge --reactor ./tcp_listener.wasm main 0 0
```

Set up a client on your localhost with [ncat](https://nmap.org/ncat).

Send any message, then send EOF with <ctrl+D>. The server would send back the reversed message.

For example, if the client send message "hello", the client would receive the response "olleh".

```
$ ncat -v 127.0.0.1 1234
Ncat: Version 7.91 ( https://nmap.org/ncat )
Ncat: Connected to 127.0.0.1:1234.
hello

olleh
```