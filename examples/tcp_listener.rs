use std::io::{Read, Write};

#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(not(feature = "std"))]
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
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
    println!("new connection at {}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    handle_client(listener.accept()?.0)
}
