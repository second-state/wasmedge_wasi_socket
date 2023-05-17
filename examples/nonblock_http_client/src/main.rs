use httparse::{Response, EMPTY_HEADER};
use std::io::{self, Read, Write};
use std::str::from_utf8;
use wasmedge_wasi_socket::TcpStream;

fn main() {
    let req = "GET /get HTTP/1.0\n\n";
    let mut first_connection = TcpStream::connect("httpbin.org:80").unwrap();
    first_connection.set_nonblocking(true).unwrap();
    first_connection.write_all(req.as_bytes()).unwrap();

    let mut second_connection = TcpStream::connect("httpbin.org:80").unwrap();
    second_connection.set_nonblocking(true).unwrap();
    second_connection.write_all(req.as_bytes()).unwrap();

    let mut first_buf = vec![0; 4096];
    let mut first_bytes_read = 0;
    let mut second_buf = vec![0; 4096];
    let mut second_bytes_read = 0;
    let mut first_complete = false;
    let mut second_complete = false;

    loop {
        if !first_complete {
            match read_data(&mut first_connection, &mut first_buf, first_bytes_read) {
                Ok((bytes_read, false)) => {
                    first_bytes_read = bytes_read;
                }
                Ok((bytes_read, true)) => {
                    println!("First connection completed");
                    if bytes_read != 0 {
                        parse_data(&first_buf, bytes_read);
                    }
                    first_complete = true;
                }
                Err(e) => {
                    println!("First connection error: {}", e);
                    first_complete = true;
                }
            }
        }
        if !second_complete {
            match read_data(&mut second_connection, &mut second_buf, second_bytes_read) {
                Ok((bytes_read, false)) => {
                    second_bytes_read = bytes_read;
                }
                Ok((bytes_read, true)) => {
                    println!("Second connection completed");
                    if bytes_read != 0 {
                        parse_data(&second_buf, bytes_read);
                    }
                    second_complete = true;
                }
                Err(e) => {
                    println!("Second connection error: {}", e);
                    second_complete = true;
                }
            }
        }
        if first_complete && second_complete {
            break;
        }
    }
}

fn read_data(
    connection: &mut TcpStream,
    data: &mut Vec<u8>,
    bytes_read: usize,
) -> io::Result<(usize, bool)> {
    let mut bytes_read = bytes_read;
    match connection.read(&mut data[bytes_read..]) {
        Ok(0) => {
            return Ok((bytes_read, true));
        }
        Ok(n) => {
            bytes_read += n;
            if bytes_read == data.len() {
                data.resize(data.len() + 1024, 0);
            }
        }
        Err(ref err) if would_block(err) => {
            let mut headers = [EMPTY_HEADER; 64];
            let mut response = Response::new(&mut headers[..]);
            match Response::parse(&mut response, &data[..bytes_read]) {
                Ok(n) => {
                    if n.is_partial() {
                        return Ok((bytes_read, false));
                    } else {
                        return Ok((n.unwrap(), true));
                    }
                }
                Err(_) => {
                    return Ok((bytes_read, false));
                }
            }
        }
        Err(ref err) if interrupted(err) => {
            return Ok((bytes_read, false));
        }
        Err(err) => {
            return Err(err);
        }
    }
    Ok((bytes_read, false))
}

fn parse_data(data: &[u8], len: usize) {
    let mut headers = [EMPTY_HEADER; 64];
    let mut response = Response::new(&mut headers[..]);
    let n = Response::parse(&mut response, &data[..len]).unwrap();
    println!("Header:");
    for header in headers {
        if !header.name.is_empty() {
            println!("{}: {:?}", header.name, from_utf8(header.value).unwrap());
        }
    }
    println!("\nBody:\n{}", from_utf8(&data[n.unwrap()..]).unwrap());
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
