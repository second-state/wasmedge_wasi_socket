use std::io::{Read, Write};
use std::net::SocketAddr;

#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(not(feature = "std"))]
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_client((mut stream, addr): (TcpStream, SocketAddr)) -> std::io::Result<()> {
    let local_addr = stream.local_addr()?;
    println!("{} <-> {}", addr.to_string(), local_addr);
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    println!("get message: {}", buf);
    println!("sendback reversed message...");
    stream.write(&buf.chars().rev().collect::<String>().into_bytes())?;

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or(1234.to_string());
    println!("listening at 127.0.0.1:{}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    handle_client(listener.accept()?)
}
