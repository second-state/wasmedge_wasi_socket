use wasmedge_wasi_socket::{
    socket::{AddressFamily, Socket, SocketType},
    ToSocketAddrs,
};

fn main() {
    let s = Socket::new(AddressFamily::Inet4, SocketType::Stream).unwrap();
    let device = s.device().unwrap();
    assert!(device.is_none());
    s.bind_device(Some(b"lo")).unwrap();
    let device = s.device().unwrap();
    assert!(device.is_some());
    assert_eq!(device.unwrap(), b"lo");
    let addr = "8.8.8.8:53".to_socket_addrs().unwrap().next().unwrap();

    let recv_timeout = s.get_recv_timeout().unwrap();
    println!("recv_timeout {:?}", recv_timeout);
    let send_timeout = s.get_send_timeout().unwrap();
    println!("send_timeout {:?}", send_timeout);

    let snd_timeout = std::time::Duration::from_secs(1);
    let rcv_timeout = std::time::Duration::from_secs(1);

    s.set_recv_timeout(Some(snd_timeout)).unwrap();
    s.set_send_timeout(Some(rcv_timeout)).unwrap();

    let recv_timeout = s.get_recv_timeout().unwrap();
    println!("recv_timeout {:?}", recv_timeout);
    assert_eq!(recv_timeout, Some(rcv_timeout));
    let send_timeout = s.get_send_timeout().unwrap();
    println!("send_timeout {:?}", send_timeout);
    assert_eq!(send_timeout, Some(snd_timeout));

    println!("start connect {addr}");
    assert!(s.connect(&addr).is_err());
}
