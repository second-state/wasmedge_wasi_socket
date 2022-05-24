use std::io::Write;
use wasmedge_wasi_socket::{Shutdown, TcpStream};

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1234".to_string());
    println!("connect to 127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    println!("sending hello message...");
    stream.write(b"hello")?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
