#[cfg(not(feature = "std"))]
use ssvm_wasm_socket::{Shutdown, TcpListener, TcpStream};
use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    println!("{}", buf);

    stream.write(&buf.chars().rev().collect::<String>().into_bytes())?;

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or(0.to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    handle_client(listener.accept()?.0)
}
