use wasmedge_wasi_socket::{SocketAddr, ToSocketAddrs};

fn main() -> std::io::Result<()> {
    // ip + port
    let mut addr = ("127.0.0.1", 3000).to_socket_addrs().unwrap();
    assert_eq!(addr.next(), Some(SocketAddr::from(([127, 0, 0, 1], 3000))));
    assert!(addr.next().is_none());

    // ip
    let mut addr = ("127.0.0.1:3000").to_socket_addrs().unwrap();
    assert_eq!(addr.next(), Some(SocketAddr::from(([127, 0, 0, 1], 3000))));
    assert!(addr.next().is_none());

    // from slice
    let addr1 = SocketAddr::from(([0, 0, 0, 0], 80));
    let addr2 = SocketAddr::from(([127, 0, 0, 1], 443));
    let addrs = vec![addr1, addr2];

    let mut addrs_iter = (&addrs[..]).to_socket_addrs().unwrap();

    assert_eq!(Some(addr1), addrs_iter.next());
    assert_eq!(Some(addr2), addrs_iter.next());
    assert!(addrs_iter.next().is_none());

    // missing port
    let err = "127.0.0.1".to_socket_addrs().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    // unable to resolve
    assert!("foo:443".to_socket_addrs().is_err());

    // dns
    let mut addr = ("localhost:3000").to_socket_addrs().unwrap();
    assert_eq!(addr.next(), Some(SocketAddr::from(([127, 0, 0, 1], 3000))));

    Ok(())
}
