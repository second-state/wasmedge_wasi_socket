use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;
use wasmedge_wasi_socket::{Shutdown, TcpStream};

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1234".to_string());
    println!("connect to 127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    stream.set_nonblocking(true)?;
    println!("sending hello message");
    stream.write(b"Hello\n")?;

    loop {
        let mut buf = [0; 128];
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("server closed connection");
                break;
            }
            Ok(size) => {
                let buf = &mut buf[..size];
                println!("get response: {}", String::from_utf8_lossy(buf));
                stream.shutdown(Shutdown::Both)?;
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("no data available, wait for 500ms");
                    sleep(Duration::from_millis(500));
                } else {
                    return Err(e);
                }
            }
        };
    }

    Ok(())
}
