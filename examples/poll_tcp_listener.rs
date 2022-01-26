// Port from https://github.com/tokio-rs/mio/blob/master/examples/tcp_server.rs

#[cfg(target_os = "wasi")]
use std::collections::HashMap;
#[cfg(target_os = "wasi")]
use std::io::{self, Read, Write};
#[cfg(target_os = "wasi")]
use std::str::from_utf8;
#[cfg(target_os = "wasi")]
use wasmedge_wasi_socket::poll::{Event, Interest, Poll, Token};
#[cfg(target_os = "wasi")]
use wasmedge_wasi_socket::{TcpListener, TcpStream};

#[cfg(target_os = "wasi")]
const DATA: &[u8] = b"Hello world!\n";

fn main() -> std::io::Result<()> {
    #[cfg(not(target_os = "wasi"))]
    {
        println!("This example is only available on WASI");
        return Ok(());
    }

    #[cfg(target_os = "wasi")]
    {
        let mut poll = Poll::new();
        let listener = TcpListener::bind("127.0.0.1:1234")?;
        let mut connections = HashMap::new();
        const SERVER: Token = Token(0);
        let mut unique_token = Token(SERVER.0 + 1);

        // Currently accept does not support non-blocking :(
        let (stream, addr) = listener.accept().unwrap();
        println!("Accepted connection from: {}", addr);
        let token = unique_token.add();
        poll.register(&stream, token, Interest::Both);
        connections.insert(token, stream);

        loop {
            let events = poll.poll().unwrap();

            for event in events {
                let done = if let Some(connection) = connections.get_mut(&event.token) {
                    handle_connection(&mut poll, connection, &event)?
                } else {
                    false
                };
                if done {
                    if let Some(connection) = connections.remove(&event.token) {
                        poll.unregister(&connection);
                    }

                    let (stream, addr) = listener.accept().unwrap();
                    println!("Accepted connection from: {}", addr);
                    let token = unique_token.add();
                    poll.register(&stream, token, Interest::Both);
                    connections.insert(token, stream);
                }
            }
        }
    }
}

#[cfg(target_os = "wasi")]
fn handle_connection(
    poll: &mut Poll,
    connection: &mut TcpStream,
    event: &Event,
) -> io::Result<bool> {
    if event.is_writable() {
        match connection.write(DATA) {
            Ok(n) if n < DATA.len() => return Err(io::ErrorKind::WriteZero.into()),
            Ok(_) => {
                poll.reregister(connection, event.token, Interest::Read);
            }
            // Seems that socket is not non-blocking, so we will not get would_block error forever :(
            Err(ref err) if would_block(err) => {}
            Err(ref err) if interrupted(err) => return handle_connection(poll, connection, event),
            Err(err) => return Err(err),
        }
    }

    if event.is_readable() {
        let mut connection_closed = false;
        let mut received_data = vec![0; 4096];
        let mut bytes_read = 0;
        loop {
            match connection.read(&mut received_data[bytes_read..]) {
                Ok(0) => {
                    connection_closed = true;
                    break;
                }
                Ok(n) => {
                    bytes_read += n;
                    if bytes_read == received_data.len() {
                        received_data.resize(received_data.len() + 1024, 0);
                    }
                }
                Err(ref err) if would_block(err) => break,
                Err(ref err) if interrupted(err) => continue,
                Err(err) => return Err(err),
            }
        }

        if bytes_read != 0 {
            let received_data = &received_data[..bytes_read];
            if let Ok(str_buf) = from_utf8(received_data) {
                println!("Received data: {}", str_buf.trim_end());
            } else {
                println!("Received (none UTF-8) data: {:?}", received_data);
            }
        }

        if connection_closed {
            println!("Connection closed");
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(target_os = "wasi")]
fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

#[cfg(target_os = "wasi")]
fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
