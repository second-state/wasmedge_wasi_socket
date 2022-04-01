use std::io::{self, Read, Write};
use std::vec;
use wasmedge_wasi_socket::poll;
use wasmedge_wasi_socket::{TcpListener, TcpStream};

const DATA: &[u8] = b"Hello world!\n";

enum NetConn {
    Server(TcpListener),
    Client(TcpStream),
}

struct Connects {
    inner: Vec<Option<NetConn>>,
}

impl Connects {
    fn new() -> Self {
        Connects { inner: vec![] }
    }

    fn next(&mut self) -> usize {
        for (i, v) in self.inner.iter_mut().enumerate() {
            if v.is_none() {
                return i;
            }
        }
        self.inner.push(None);
        self.inner.len() - 1
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut NetConn> {
        if let Some(x) = self.inner.get_mut(id)? {
            Some(x)
        } else {
            None
        }
    }

    fn slice(&self) -> &[Option<NetConn>] {
        self.inner.as_slice()
    }

    fn add(&mut self, conn: NetConn) -> usize {
        let next_id = self.next();
        let _ = self.inner[next_id].insert(conn);
        next_id
    }

    fn remove(&mut self, id: usize) -> Option<NetConn> {
        println!("remove conn[{}]", id);
        self.inner.get_mut(id).and_then(|v| v.take())
    }
}

fn connects_to_subscriptions(connects: &Connects) -> Vec<poll::Subscription> {
    let mut subscriptions = vec![];
    for (i, conn) in connects.slice().iter().enumerate() {
        if let Some(conn) = conn {
            match conn {
                NetConn::Server(s) => {
                    subscriptions.push(poll::Subscription::io(i as u64, s, true, false, None));
                }
                NetConn::Client(s) => {
                    subscriptions.push(poll::Subscription::io(i as u64, s, true, false, None));
                }
            }
        }
    }
    subscriptions
}

fn main() -> std::io::Result<()> {
    let mut connects = Connects::new();
    let server = TcpListener::bind("127.0.0.1:1234", true)?;
    connects.add(NetConn::Server(server));

    loop {
        let subs = connects_to_subscriptions(&connects);
        let events = poll::poll(&subs)?;

        for event in events {
            let conn_id = event.userdata as usize;
            match connects.get_mut(conn_id) {
                Some(NetConn::Server(server)) => match event.event_type {
                    poll::EventType::Timeout => unreachable!(),
                    poll::EventType::Error(e) => {
                        return Err(e);
                    }
                    poll::EventType::Read => {
                        let (mut tcp_client, addr) = server.accept(true)?;
                        println!("accept from {}", addr);

                        match tcp_client.write(DATA) {
                            Ok(n) if n < DATA.len() => {
                                println!(
                                    "write hello error: {}",
                                    io::Error::from(io::ErrorKind::WriteZero)
                                );
                                continue;
                            }
                            Ok(_) => {}
                            Err(ref err) if would_block(err) => {}
                            Err(ref err) if interrupted(err) => {}
                            Err(err) => {
                                println!("write hello error: {}", err);
                                continue;
                            }
                        }

                        let id = connects.add(NetConn::Client(tcp_client));
                        println!("add conn[{}]", id);
                    }
                    poll::EventType::Write => unreachable!(),
                },
                Some(NetConn::Client(client)) => {
                    match event.event_type {
                        poll::EventType::Timeout => {
                            // if Subscription timeout is not None.
                            unreachable!()
                        }
                        poll::EventType::Error(e) => {
                            println!("tcp_client[{}] recv a io error: {}", conn_id, e);
                            connects.remove(conn_id);
                        }
                        poll::EventType::Read => match handle_connection_read(client) {
                            Ok(true) => {
                                println!("tcp_client[{}] is closed", conn_id);
                                connects.remove(conn_id);
                            }
                            Err(e) => {
                                println!("tcp_client[{}] recv a io error: {}", conn_id, e);
                                connects.remove(conn_id);
                            }
                            _ => {}
                        },
                        poll::EventType::Write => unreachable!(),
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_connection_read(connection: &mut TcpStream) -> io::Result<bool> {
    let mut connection_closed = false;
    let mut received_buff = [0u8; 2048];

    let mut received_data = Vec::with_capacity(2048);
    loop {
        match connection.read(&mut received_buff) {
            Ok(0) => {
                connection_closed = true;
                break;
            }
            Ok(n) => {
                received_data.extend_from_slice(&received_buff[0..n]);
            }
            Err(ref err) if would_block(err) => break,
            Err(ref err) if interrupted(err) => continue,
            Err(err) => return Err(err),
        }
    }

    if !received_data.is_empty() {
        if let Ok(str_buf) = std::str::from_utf8(&received_data) {
            println!("Received data: {}", str_buf.trim_end());
        } else {
            println!("Received (none UTF-8) data: {:?}", received_data);
        }
    }

    if connection_closed {
        return Ok(true);
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
