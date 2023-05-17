pub mod poll;
pub mod socket;
#[cfg(feature = "wasi_poll")]
pub mod wasi_poll;
#[cfg(not(feature = "wasi_poll"))]
mod wasi_poll;
pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use std::{
    io::{self, Read, Write},
    net::{SocketAddrV4, SocketAddrV6},
    os::wasi::prelude::{AsRawFd, FromRawFd, IntoRawFd},
};

#[derive(Debug)]
pub struct TcpStream {
    s: socket::Socket,
}

impl AsRef<socket::Socket> for TcpStream {
    fn as_ref(&self) -> &socket::Socket {
        &self.s
    }
}

impl AsMut<socket::Socket> for TcpStream {
    fn as_mut(&mut self) -> &mut socket::Socket {
        &mut self.s
    }
}

#[derive(Debug)]
pub struct TcpListener {
    s: socket::Socket,
    pub address: std::io::Result<SocketAddr>,
    pub port: Option<u16>,
}

impl AsRef<socket::Socket> for TcpListener {
    fn as_ref(&self) -> &socket::Socket {
        &self.s
    }
}

impl AsMut<socket::Socket> for TcpListener {
    fn as_mut(&mut self) -> &mut socket::Socket {
        &mut self.s
    }
}

#[derive(Debug)]
pub struct UdpSocket {
    s: socket::Socket,
}

impl AsRef<socket::Socket> for UdpSocket {
    fn as_ref(&self) -> &socket::Socket {
        &self.s
    }
}

impl AsMut<socket::Socket> for UdpSocket {
    fn as_mut(&mut self) -> &mut socket::Socket {
        &mut self.s
    }
}

impl TcpStream {
    /// Create TCP socket and connect to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn connect<A: ToSocketAddrs>(addrs: A) -> io::Result<TcpStream> {
        let mut last_error = io::Error::from(io::ErrorKind::ConnectionRefused);
        let addrs = addrs.to_socket_addrs()?;

        let connect = |addrs| {
            let addr_family = socket::AddressFamily::from(&addrs);
            let s = socket::Socket::new(addr_family, socket::SocketType::Stream)?;
            s.connect(&addrs)?;
            Ok(s)
        };

        for addr in addrs {
            match connect(addr) {
                Ok(s) => return Ok(TcpStream { s }),
                Err(e) => last_error = e,
            }
        }
        return Err(last_error);
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.s.shutdown(how)
    }

    /// Get peer address.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_peer()
    }

    /// Get local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_local()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.s.set_nonblocking(nonblocking)
    }

    pub fn new(s: socket::Socket) -> Self {
        Self { s }
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> std::os::wasi::prelude::RawFd {
        self.s.as_raw_fd()
    }
}

impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> std::os::wasi::prelude::RawFd {
        self.s.into_raw_fd()
    }
}

impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: std::os::wasi::prelude::RawFd) -> Self {
        Self {
            s: socket::Socket::from_raw_fd(fd),
        }
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.s.recv(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.s.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.s.recv(buf)
    }
}

impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.s.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl From<socket::Socket> for TcpStream {
    fn from(s: socket::Socket) -> Self {
        TcpStream { s }
    }
}

impl TcpListener {
    /// Create TCP socket and bind to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn bind<A: ToSocketAddrs>(addrs: A, nonblocking: bool) -> io::Result<TcpListener> {
        let mut last_error = io::Error::from(io::ErrorKind::Other);
        let addrs = addrs.to_socket_addrs()?;

        let bind = |addrs, nonblocking| {
            let addr_family = socket::AddressFamily::from(&addrs);
            let s = socket::Socket::new(addr_family, socket::SocketType::Stream)?;
            s.setsockopt(
                socket::SocketOptLevel::SolSocket,
                socket::SocketOptName::SoReuseaddr,
                1i32,
            )?;
            s.bind(&addrs)?;
            s.listen(128)?;
            s.set_nonblocking(nonblocking)?;

            let port = addrs.port();
            Ok(TcpListener {
                s,
                address: Ok(addrs),
                port: Some(port),
            })
        };

        for addr in addrs {
            match bind(addr, nonblocking) {
                Ok(tcp_listener) => return Ok(tcp_listener),
                Err(e) => last_error = e,
            }
        }

        return Err(last_error);
    }

    /// Accept incoming connections with given file descriptor flags.
    pub fn accept(&self, nonblocking: bool) -> io::Result<(TcpStream, SocketAddr)> {
        let s = self.s.accept(nonblocking)?;
        let stream = TcpStream { s };
        let peer = stream.peer_addr()?;
        Ok((stream, peer))
    }

    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// Get local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_local()
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> std::os::wasi::prelude::RawFd {
        self.s.as_raw_fd()
    }
}

impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> std::os::wasi::prelude::RawFd {
        self.s.into_raw_fd()
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: std::os::wasi::prelude::RawFd) -> Self {
        let s: socket::Socket = FromRawFd::from_raw_fd(fd);
        match s.get_local() {
            Ok(address) => {
                let port = address.port();
                TcpListener {
                    s,
                    address: Ok(address),
                    port: Some(port),
                }
            }
            Err(error) => TcpListener {
                s,
                address: Err(error),
                port: None,
            },
        }
    }
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept(false).map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

impl UdpSocket {
    /// Create UDP socket and bind to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> io::Result<UdpSocket> {
        let mut last_error = io::Error::from(io::ErrorKind::Other);
        let addrs = addrs.to_socket_addrs()?;

        let bind = |addrs| {
            let addr_family = socket::AddressFamily::from(&addrs);
            let s = socket::Socket::new(addr_family, socket::SocketType::Datagram)?;
            s.bind(&addrs)?;
            Ok(UdpSocket { s })
        };

        for addr in addrs {
            match bind(addr) {
                Ok(udp) => return Ok(udp),
                Err(e) => last_error = e,
            }
        }

        return Err(last_error);
    }
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.s.recv_from(buf)
    }
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "No address."))?;

        self.s.send_to(buf, addr)
    }
}

impl AsRawFd for UdpSocket {
    fn as_raw_fd(&self) -> std::os::wasi::prelude::RawFd {
        self.s.as_raw_fd()
    }
}

#[cfg(not(feature = "built-in-dns"))]
pub fn nslookup(node: &str, service: &str) -> std::io::Result<Vec<SocketAddr>> {
    use socket::WasiAddrinfo;
    let hints: WasiAddrinfo = WasiAddrinfo::default();
    let mut sockaddrs = Vec::new();
    let mut sockbuffs = Vec::new();
    let mut ai_canonnames = Vec::new();
    let addrinfos = WasiAddrinfo::get_addrinfo(
        &node,
        &service,
        &hints,
        10,
        &mut sockaddrs,
        &mut sockbuffs,
        &mut ai_canonnames,
    )?;

    let mut r_addrs = vec![];
    for i in 0..addrinfos.len() {
        let addrinfo = &addrinfos[i];
        let sockaddr = &sockaddrs[i];
        let sockbuff = &sockbuffs[i];

        if addrinfo.ai_addrlen == 0 {
            continue;
        }

        let addr = match sockaddr.family {
            socket::AddressFamily::Unspec => {
                //unimplemented!("not support unspec")
                continue;
            }
            socket::AddressFamily::Inet4 => {
                let port_buf = [sockbuff[0], sockbuff[1]];
                let port = u16::from_be_bytes(port_buf);
                let ip = Ipv4Addr::new(sockbuff[2], sockbuff[3], sockbuff[4], sockbuff[5]);
                SocketAddr::V4(SocketAddrV4::new(ip, port))
            }
            socket::AddressFamily::Inet6 => {
                //unimplemented!("not support IPv6")
                continue;
            }
        };

        r_addrs.push(addr);
    }
    Ok(r_addrs)
}

#[cfg(feature = "built-in-dns")]
pub fn nslookup(node: &str, _service: &str) -> std::io::Result<Vec<SocketAddr>> {
    let dns_server = std::env::var("DNS_SERVER").unwrap_or("8.8.8.8:53".into());
    let mut conn = TcpStream::connect(dns_server)?;
    let timeout = std::time::Duration::from_secs(1);
    let _ignore = conn.as_mut().set_send_timeout(Some(timeout));
    let _ignore = conn.as_mut().set_recv_timeout(Some(timeout));

    if node == "localhost" {
        return ("127.0.0.1", 0u16).to_socket_addrs().map(|v| v.collect());
    }
    let r = resolve::<_, Ipv4Addr>(&mut conn, node)
        .unwrap_or_default()
        .into_iter()
        .map(|addr| (addr, 0).into())
        .collect::<Vec<SocketAddr>>();
    if r.is_empty() {
        // __WASI_ERRNO_AINODATA
        Err(std::io::Error::from_raw_os_error(83))
    } else {
        Ok(r)
    }
}

pub trait ToQType: Sized {
    fn q_type() -> dns_parser::QueryType;

    fn from_rr(rr: dns_parser::RData) -> Option<Self>;
}

impl ToQType for Ipv4Addr {
    fn q_type() -> dns_parser::QueryType {
        dns_parser::QueryType::A
    }

    fn from_rr(rr: dns_parser::RData) -> Option<Self> {
        if let dns_parser::RData::A(ip) = rr {
            Some(ip.0)
        } else {
            None
        }
    }
}

impl ToQType for Ipv6Addr {
    fn q_type() -> dns_parser::QueryType {
        dns_parser::QueryType::AAAA
    }

    fn from_rr(rr: dns_parser::RData) -> Option<Self> {
        if let dns_parser::RData::AAAA(ip) = rr {
            Some(ip.0)
        } else {
            None
        }
    }
}

pub fn resolve<S: Write + Read, T: ToQType>(
    conn: &mut S,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    use dns_parser::QueryClass;
    use dns_parser::{Builder, Packet, ResponseCode};
    let id = rand::random();
    let mut builder = Builder::new_query(id, true);
    builder.add_question(name, false, T::q_type(), QueryClass::IN);
    let packet = builder.build().map_err(|_| "truncated packet")?;
    let mut psize = (packet.len() as u16).to_be_bytes();

    conn.write_all(&psize[..])?;
    conn.write_all(&packet)?;

    let n = conn.read(&mut psize)?;
    if n < 2 {
        return Err("Partial packet received".into());
    }

    let psize = u16::from_be_bytes(psize) as usize;
    let mut buf = vec![0u8; psize];
    let n = conn.read(&mut buf[0..psize])?;
    if n != psize {
        return Err("Partial packet received".into());
    }

    let pkt = Packet::parse(&buf)?;

    if pkt.header.id != id {
        return Err("Illegal id".into());
    }

    if pkt.header.response_code != ResponseCode::NoError {
        return Err(pkt.header.response_code.into());
    }

    let mut address = vec![];
    for ans in pkt.answers {
        if let Some(addr) = T::from_rr(ans.data) {
            address.push(addr);
        }
    }
    Ok(address)
}

/*
Implement ToScoketAddrs using nslookup, so that DNS can be resolved in wasi.
*/
pub trait ToSocketAddrs {
    type Iter: Iterator<Item = SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<std::net::Ipv4Addr>() {
            let addr = std::net::SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter());
        }
        if let Ok(addr) = host.parse::<std::net::Ipv6Addr>() {
            let addr = std::net::SocketAddrV6::new(addr, port, 0, 0);
            return Ok(vec![SocketAddr::V6(addr)].into_iter());
        }
        let v: Vec<_> = nslookup(host, "http")?
            .into_iter()
            .map(|mut a| {
                a.set_port(port);
                a
            })
            .collect();
        Ok(v.into_iter())
    }
}

impl ToSocketAddrs for (String, u16) {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        (&*self.0, self.1).to_socket_addrs()
    }
}

// accepts strings like 'localhost:12345'
impl ToSocketAddrs for str {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        let host_and_port = self.split(":").collect::<Vec<&str>>();
        if host_and_port.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid socket address",
            ));
        }
        let host = host_and_port[0];
        let port = str::parse::<u16>(host_and_port[1]).map_err(|_e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid port value")
        })?;
        let mut addrs = nslookup(host, "http")?;
        for addr in addrs.iter_mut() {
            addr.set_port(port);
        }
        Ok(addrs.into_iter())
    }
}

impl ToSocketAddrs for String {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        (&**self).to_socket_addrs()
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = std::iter::Cloned<std::slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}
