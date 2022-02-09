use wasmedge_wasi_socket::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let port = std::env::var("PORT").unwrap_or(0.to_string());
    let addr = format!("127.0.0.1:{}", port);

    socket.send_to(b"hello", &addr)?;

    let mut buf = [0; 128];
    let (size, _) = socket.recv_from(&mut buf)?;

    match std::str::from_utf8(&buf[..size]) {
        Ok(s) => Ok(println!("{}", s)),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}
