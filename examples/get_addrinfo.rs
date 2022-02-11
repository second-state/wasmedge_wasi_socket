use wasmedge_wasi_socket::WasiAddrinfo;

fn main() {
    let node = String::from("google.com");
    let service = String::from("http");
    let hints: WasiAddrinfo = WasiAddrinfo::default();
    let mut sockaddr = Vec::new();
    let mut sockbuff = Vec::new();
    let mut ai_canonname = Vec::new();
    let addrinfo = WasiAddrinfo::get_addrinfo(
        &node,
        &service,
        &hints,
        10,
        &mut sockaddr,
        &mut sockbuff,
        &mut ai_canonname,
    )
    .unwrap();
    for i in 0..addrinfo.len() {
        if addrinfo[i].ai_addrlen.ne(&0) {
            println!("{:?}", sockbuff[i]);
        }
    }
}
