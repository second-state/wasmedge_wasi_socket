use http_req::request;

fn main() {
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("http://18.235.124.214/get", &mut writer).unwrap();

    println!("GET");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());
    println!("{}", String::from_utf8_lossy(&writer));

    let mut writer = Vec::new(); //container for body of a response
    const BODY: &[u8; 27] = b"field1=value1&field2=value2";
    // let res = request::post("https://httpbin.org/post", BODY, &mut writer).unwrap();
    // no https , no dns
    let res = request::post("http://18.235.124.214/post", BODY, &mut writer).unwrap();

    println!("POST");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());
    println!("{}", String::from_utf8_lossy(&writer));
}
