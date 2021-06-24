use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(not(feature = "std"))]
use w13e_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    println!("{}", buf);

    stream.write(&buf.chars().rev().collect::<String>().into_bytes())?;

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    handle_client(listener.accept()?.0)
}
