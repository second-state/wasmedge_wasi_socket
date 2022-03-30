use parsed::http::{Header, Response};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use wasmedge_wasi_socket::poll::{Event, Interest, Poll, Token};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream, FDFLAGS_NONBLOCK};

struct Handler {
    pub response: Option<String>,
}

impl Handler {
    pub fn new() -> Handler {
        Handler { response: None }
    }
}

fn main() -> std::io::Result<()> {
    let mut poll = Poll::new();
    let server = TcpListener::bind("127.0.0.1:1234", true)?;
    println!("Listening on 127.0.0.1:1234");
    let mut connections = HashMap::new();
    let mut handlers = HashMap::new();
    const SERVER: Token = Token(0);
    let mut unique_token = Token(SERVER.0 + 1);

    poll.register(&server, SERVER, Interest::Read);

    loop {
        let events = poll.poll().unwrap();

        for event in events {
            match event.token {
                SERVER => loop {
                    let (connection, address) = match server.accept(FDFLAGS_NONBLOCK) {
                        Ok((connection, address)) => (connection, address),
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => panic!("accept error: {}", e),
                    };

                    println!("Accepted connection from: {}", address);

                    let token = unique_token.add();
                    poll.register(&connection, token, Interest::Read);
                    connections.insert(token, connection);
                },
                token => {
                    let done = if let Some(connection) = connections.get_mut(&token) {
                        let handler = match handlers.get_mut(&token) {
                            Some(handler) => handler,
                            None => {
                                let handler = Handler::new();
                                handlers.insert(token, handler);
                                handlers.get_mut(&token).unwrap()
                            }
                        };
                        handle_connection(&mut poll, connection, handler, &event)?
                    } else {
                        false
                    };
                    if done {
                        if let Some(connection) = connections.remove(&token) {
                            connection.shutdown(Shutdown::Both)?;
                            poll.unregister(&connection);
                            handlers.remove(&token);
                        }
                    }
                }
            }
        }
    }
}

fn handle_connection(
    poll: &mut Poll,
    connection: &mut TcpStream,
    handler: &mut Handler,
    event: &Event,
) -> io::Result<bool> {
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
                Err(ref err) if would_block(err) => {
                    if bytes_read != 0 {
                        let received_data = &received_data[..bytes_read];
                        if !received_data.ends_with(b"\r\n\r\n") {
                            continue;
                        }
                        let mut bs: parsed::stream::ByteStream =
                            match String::from_utf8(received_data.to_vec()) {
                                Ok(s) => s,
                                Err(_) => {
                                    continue;
                                }
                            }
                            .into();
                        let req = match parsed::http::parse_http_request(&mut bs) {
                            Some(req) => req,
                            None => {
                                break;
                            }
                        };
                        for header in req.headers.iter() {
                            if header.name.eq("Conntent-Length") {
                                let content_length = header.value.parse::<usize>().unwrap();
                                if content_length > received_data.len() {
                                    return Ok(true);
                                }
                            }
                        }
                        println!(
                            "{:?} request: {:?} {:?}",
                            connection.peer_addr().unwrap(),
                            req.method,
                            req.path
                        );
                        let res = Response {
                            protocol: "HTTP/1.1".to_string(),
                            code: 200,
                            message: "OK".to_string(),
                            headers: vec![
                                Header {
                                    name: "Content-Length".to_string(),
                                    value: req.content.len().to_string(),
                                },
                                Header {
                                    name: "Connection".to_string(),
                                    value: "close".to_string(),
                                },
                            ],
                            content: req.content,
                        };

                        handler.response = Some(res.into());

                        poll.reregister(connection, event.token, Interest::Write);
                        break;
                    } else {
                        println!("Empty request");
                        return Ok(true);
                    }
                }
                Err(ref err) if interrupted(err) => continue,
                Err(err) => return Err(err),
            }
        }

        if connection_closed {
            println!("Connection closed");
            return Ok(true);
        }
    }

    if event.is_writable() && handler.response.is_some() {
        let resp = handler.response.clone().unwrap();
        match connection.write(resp.as_bytes()) {
            Ok(n) if n < resp.len() => return Err(io::ErrorKind::WriteZero.into()),
            Ok(_) => {
                return Ok(true);
            }
            Err(ref err) if would_block(err) => {}
            Err(ref err) if interrupted(err) => {
                return handle_connection(poll, connection, handler, event)
            }
            Err(err) => return Err(err),
        }
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
