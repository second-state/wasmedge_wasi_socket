use wasmedge_http_req::request;
use wasmedge_wasi_socket::WasiAddrinfo;

fn main() {
    // DNS query
    let hints: WasiAddrinfo = WasiAddrinfo::default();
    let mut sockaddr = Vec::new();
    let mut sockbuff = Vec::new();
    let mut ai_canonname = Vec::new();
    let addrinfo = WasiAddrinfo::get_addrinfo(
        "localhost",
        "1234",
        &hints,
        10,
        &mut sockaddr,
        &mut sockbuff,
        &mut ai_canonname,
    )
    .unwrap();

    // There should always a result for localhost
    assert!(!addrinfo.is_empty());
    // Get first result and check if is IPv4 address
    if addrinfo[0].ai_family.is_v4() && addrinfo[0].ai_addrlen.ne(&0) {
        let port = u16::from_be_bytes(sockbuff[0][0..2].try_into().unwrap());
        let ip = format!(
            "{}.{}.{}.{}",
            sockbuff[0][2], sockbuff[0][3], sockbuff[0][4], sockbuff[0][5]
        );

        let mut writer = Vec::new(); //container for body of a response
        let res = request::get(format!("http://{}:{}/get", ip, port), &mut writer).unwrap();
        println!("GET");
        println!("Status: {} {}", res.status_code(), res.reason());
        println!("Headers {}", res.headers());
        println!("{}", String::from_utf8_lossy(&writer));

        writer.clear();
        const BODY: &[u8; 42] = b"{\"field1\" : \"value1\", \"field2\" : \"value2\"}";
        let res = request::post(format!("http://{}:{}/post", ip, port), BODY, &mut writer).unwrap();

        println!("POST");
        println!("Status: {} {}", res.status_code(), res.reason());
        println!("Headers {}", res.headers());
        println!("{}", String::from_utf8_lossy(&writer));
    }
}
