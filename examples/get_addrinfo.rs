use wasmedge_wasi_socket::nslookup;

fn main() {
    let addrs = nslookup("google.com", "http").unwrap();
    for addr in addrs {
        println!("{:?}",addr);
    }
}
