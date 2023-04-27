use wasmedge_wasi_socket::{resolve, Ipv4Addr, TcpStream};

fn main() {
    let mut code = 0;

    for name in std::env::args().skip(1) {
        let mut conn = TcpStream::connect("8.8.8.8:53").unwrap();

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
