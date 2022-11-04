pub mod poll;
pub mod socket;
#[cfg(feature = "wasi_poll")]
pub mod wasi_poll;
#[cfg(not(feature = "wasi_poll"))]
mod wasi_poll;
pub use socket::WasiAddrinfo;
pub use std::net::{IpAddr, Shutdown, SocketAddr, ToSocketAddrs};
use std::{
    io::{self, Read, Write},
    os::wasi::prelude::{AsRawFd, FromRawFd, IntoRawFd},
};

#[derive(Debug)]
pub struct TcpStream {
    s: socket::Socket,
}

#[derive(Debug)]
pub struct TcpListener {
    s: socket::Socket,
    pub address: std::io::Result<SocketAddr>,
    pub port: Option<u16>,
}

#[derive(Debug)]
pub struct UdpSocket {
    s: socket::Socket,
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

#[cfg(not(feature = "wasmedge_asyncify"))]
pub fn nslookup(node: &str, service: &str) -> std::io::Result<Vec<SocketAddr>> {
    use std::net::Ipv4Addr;
    use std::net::SocketAddrV4;

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
            #[cfg(not(feature = "wasmedge_0_9"))]
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

#[cfg(feature = "wasmedge_asyncify")]
pub fn nslookup_v4(host: &str) -> std::io::Result<Vec<std::net::Ipv4Addr>> {
    socket::lookup_ipv4(host, 10)
}

#[cfg(feature = "wasmedge_asyncify")]
pub fn nslookup_v6(host: &str) -> std::io::Result<Vec<std::net::Ipv6Addr>> {
    socket::lookup_ipv6(host, 10)
}

#[cfg(feature = "wasmedge_asyncify")]
pub fn nslookup(node: &str, service: &str) -> std::io::Result<Vec<SocketAddr>> {
    let port = match service {
        "ssh" => 22,
        "telnet" => 23,
        "smtp" => 25,
        "http" => 80,
        "https" => 443,
        _ => return Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    };
    let ipv4 = nslookup_v4(node).map(|ip_vec| {
        ip_vec
            .into_iter()
            .map(|ip| SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)))
            .collect::<Vec<SocketAddr>>()
    });
    let ipv6 = nslookup_v6(node).map(|ip_vec| {
        ip_vec
            .into_iter()
            .map(|ip| SocketAddr::V6(std::net::SocketAddrV6::new(ip, port, 0, 0)))
            .collect::<Vec<SocketAddr>>()
    });

    match (ipv4, ipv6) {
        (Ok(mut ipv4), Ok(ipv6)) => {
            ipv4.extend(ipv6);
            Ok(ipv4)
        }
        (Ok(ipv4), Err(_)) => Ok(ipv4),
        (Err(_), Ok(ipv6)) => Ok(ipv6),
        (Err(e), Err(_)) => Err(e),
    }
}
