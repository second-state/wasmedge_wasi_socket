use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpStream};
#[cfg(not(feature = "std"))]
use w13e_wasi_socket::{Shutdown, TcpStream};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1234")?;
    stream.write(b"hello")?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
