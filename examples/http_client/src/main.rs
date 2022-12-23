use http_req::request;
use wasmedge_wasi_socket::nslookup;

fn main() -> std::io::Result<()> {
    // DNS query
    let addrs = nslookup("httpbin.org", "http")?;
    assert!(!addrs.is_empty());
    // Get first result and check if is IPv4 address
    let addr = addrs[0];

    let mut writer = Vec::new(); //container for body of a response
    let res = request::get(format!("http://{}/get", addr), &mut writer).unwrap();
    println!("GET");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());
    println!("{}", String::from_utf8_lossy(&writer));

    writer.clear();
    const BODY: &[u8; 42] = b"{\"field1\" : \"value1\", \"field2\" : \"value2\"}";
    let res = request::post(format!("http://{}/post", addr), BODY, &mut writer).unwrap();

    println!("POST");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());
    println!("{}", String::from_utf8_lossy(&writer));
    Ok(())
}
