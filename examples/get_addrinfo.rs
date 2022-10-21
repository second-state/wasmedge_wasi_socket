#[cfg(not(feature = "wasmedge_asyncify"))]
use wasmedge_wasi_socket::nslookup;

#[cfg(feature = "wasmedge_asyncify")]
use wasmedge_wasi_socket::nslookup_v4;

fn main() {
    #[cfg(not(feature = "wasmedge_asyncify"))]
    let addrs = nslookup("google.com", "http").unwrap();
    #[cfg(feature = "wasmedge_asyncify")]
    let addrs = nslookup_v4("google.com").unwrap();
    for addr in addrs {
        println!("{:?}", addr);
    }
}
