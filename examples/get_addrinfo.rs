use std::net::SocketAddrV4;

use wasmedge_wasi_socket::{Ipv4Addr, SocketAddr, WasiAddrinfo};

fn main() {
    let node = "google.com";
    let service = "http";
    let hints: WasiAddrinfo = WasiAddrinfo::default();
    let mut sockaddrs = Vec::new();
    let mut sockbuffs = Vec::new();
    let mut ai_canonnames = Vec::new();
    let addrinfos = WasiAddrinfo::get_addrinfo(
        node,
        service,
        &hints,
        10,
        &mut sockaddrs,
        &mut sockbuffs,
        &mut ai_canonnames,
    )
    .unwrap();
    for i in 0..addrinfos.len() {
        let addrinfo = &addrinfos[i];
        let sockaddr = &sockaddrs[i];
        let sockbuff = &sockbuffs[i];

        if addrinfo.ai_addrlen == 0 {
            continue;
        }

        let addr = match sockaddr.family {
            wasmedge_wasi_socket::socket::AddressFamily::Inet4 => {
                let port_buf = [sockbuff[0], sockbuff[1]];
                let port = u16::from_be_bytes(port_buf);
                let ip = Ipv4Addr::new(sockbuff[2], sockbuff[3], sockbuff[4], sockbuff[5]);
                SocketAddr::V4(SocketAddrV4::new(ip, port))
            }
            wasmedge_wasi_socket::socket::AddressFamily::Inet6 => {
                // unimplemented!("not support IPv6")
                continue;
            }
        };
        println!(
            "{:?}\n{:?}\n{:?}\n{:?}\n",
            addrinfo, sockaddr, sockbuff, addr
        );
    }
}
