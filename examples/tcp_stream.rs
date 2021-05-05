use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpStream};
#[cfg(not(feature = "std"))]
use w13e_wasi_socket::{Shutdown, TcpStream};

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or(0.to_string());
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;

    stream.write(b"hello")?;

    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    println!("{}", buf);

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
