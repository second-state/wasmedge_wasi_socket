use wasmedge_wasi_socket::{resolve, Ipv4Addr, TcpStream};

fn main() {
    let mut code = 0;

    for name in std::env::args().skip(1) {
        println!("resolve {name}");
        let mut conn = TcpStream::connect("127.0.0.1:8000").unwrap();
        conn.as_ref()
            .set_recv_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();

        match resolve::<_, Ipv4Addr>(&mut conn, &name) {
            Ok(address) => {
                println!("{:#?}", address);
            }
            Err(e) => {
                eprintln!("Error resolving {:?}: {}", name, e);
                code = 1;
            }
        }
    }
    std::process::exit(code);
}
