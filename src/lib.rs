use std::ffi::CString;
use std::io::{Read, Result, Write};
pub use std::net::{Shutdown, SocketAddr, ToSocketAddrs, IpAddr, Ipv4Addr};

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn sock_open(addr_family: u8, sock_type: u8, fd: *mut u32) -> u32;
    pub fn sock_close(fd: u32);
    pub fn sock_bind(fd: u32, addr: *const u8, addr_len: u32) -> u32;
    pub fn sock_listen(fd: u32, backlog: u32) -> u32;
    pub fn sock_accept(fd: u32, fd: *mut u32) -> u32;
    pub fn sock_connect(fd: u32, addr: *const u8, addr_len: u32) -> u32;
    pub fn sock_recv(fd: u32, buf: *const u8, buf_len: usize, flags: u16, recv_len: *mut usize) -> u32;
    pub fn sock_recv_from(
        fd: u32,
        buf: *mut u8,
        buf_len: u32,
        addr: *mut u8,
        addr_len: *mut u32,
        flags: u16,
    ) -> u32;
    pub fn sock_send(fd: u32, buf: *const u8, buf_len: u32, flags: u16, send_len: *mut u32) -> u32;
    pub fn sock_send_to(
        fd: u32,
        buf: *const u8,
        buf_len: u32,
        addr: *const u8,
        addr_len: u32,
        flags: u16,
    ) -> u32;
    pub fn sock_shutdown(fd: u32, flags: u8) -> u32;
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum AddressFamily {
    Inet4,
    Inet6,
}

impl From<SocketAddr> for AddressFamily {
    fn from(addr: SocketAddr) -> AddressFamily {
        match addr {
            SocketAddr::V4(_) => AddressFamily::Inet4,
            SocketAddr::V6(_) => AddressFamily::Inet6,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum SocketType {
    Datagram,
    Stream,
}

trait AsRawFd {
    fn as_raw_fd(&self) -> u32;
}

#[derive(Copy, Clone)]
struct SocketHandle(u32);

impl AsRawFd for SocketHandle {
    fn as_raw_fd(&self) -> u32 {
        self.0
    }
}

#[non_exhaustive]
pub struct TcpStream {
    fd: SocketHandle,
}

#[non_exhaustive]
pub struct TcpListener {
    fd: SocketHandle,
}

#[non_exhaustive]
pub struct UdpSocket {
    fd: SocketHandle,
}

macro_rules! impl_as_raw_fd {
    ($($t:ident)*) => {$(
        impl AsRawFd for $t {
            fn as_raw_fd(&self) -> u32 {
                self.fd.as_raw_fd()
            }
        }
    )*};
}

impl_as_raw_fd! { TcpStream TcpListener UdpSocket }


impl TcpStream {
    /// Create TCP socket and connect to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn connect<A: ToSocketAddrs>(addrs: A) -> Result<TcpStream> {
        match addrs.to_socket_addrs()?.find_map(|addr| unsafe {
            let mut fd: u32 = 0;
            sock_open(
                AddressFamily::from(addr) as u8,
                SocketType::Stream as u8,
                &mut fd,
            );
            let addr_s = CString::new(addr.to_string()).expect("CString::new");
            sock_connect(
                fd,
                addr_s.as_ptr() as *const u8,
                addr_s.as_bytes().len() as u32,
            );
            Some(SocketHandle(fd))
        }) {
            Some(fd) => Ok(TcpStream { fd }),
            _ => Err(std::io::Error::last_os_error()),
        }
    }
    pub fn shutdown(&self, how: Shutdown) -> Result<()> {
        unsafe {
            sock_shutdown(self.as_raw_fd(), how as u8);
        }
        Ok(())
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let flags = 0;
        let mut recv_len: usize = 0;

        let ret = unsafe {
            sock_recv(self.as_raw_fd(), buf.as_mut_ptr(), buf.len(), flags, &mut recv_len);
        };
        Ok(recv_len)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let sent = unsafe {
            let mut send_len: u32 = 0;
            sock_send(
                self.as_raw_fd(),
                buf.as_ptr() as *const u8,
                buf.len() as u32,
                0,
                &mut send_len,
            )
        };
        Ok(sent as usize)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}



impl TcpListener {
    /// Create TCP socket and bind to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<TcpListener> {
        match addrs.to_socket_addrs()?.find_map(|addr| unsafe {
            let mut fd: u32 = 0;
            sock_open(
                AddressFamily::from(addr) as u8,
                SocketType::Stream as u8,
                &mut fd,
            );
            let addr_s = CString::new(addr.to_string()).expect("CString::new");
            sock_bind(
                fd,
                addr_s.as_ptr() as *const u8,
                addr_s.as_bytes().len() as u32,
            );
            sock_listen(fd, 128);
            Some(SocketHandle(fd))
        }) {
            Some(fd) => Ok(TcpListener { fd }),
            _ => Err(std::io::Error::last_os_error()),
        }
    }
    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        unsafe {
            let mut fd: u32 = 0;
            sock_accept(
                self.as_raw_fd(),
                &mut fd
            );
            let fd = SocketHandle(fd);
            Ok( (TcpStream{fd}, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)) )
        }
    }
}

impl UdpSocket {
    /// Create UDP socket and bind to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<UdpSocket> {
        match addrs.to_socket_addrs()?.find_map(|addr| unsafe {
            let mut fd: u32 = 0;
            sock_open(
                AddressFamily::from(addr) as u8,
                SocketType::Stream as u8,
                &mut fd,
            );
            let addr_s = CString::new(addr.to_string()).expect("CString::new");
            sock_bind(
                fd,
                addr_s.as_ptr() as *const u8,
                addr_s.as_bytes().len() as u32,
            );
            Some(SocketHandle(fd))
        }) {
            Some(fd) => Ok(UdpSocket { fd }),
            _ => Err(std::io::Error::last_os_error()),
        }
    }
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let mut addr_len: u32 = 0;
        let mut addr_buf = [0; 32];
        let size = unsafe {
            sock_recv_from(
                self.as_raw_fd(),
                buf.as_ptr() as *mut u8,
                buf.len() as u32,
                addr_buf.as_ptr() as *mut u8,
                &mut addr_len,
                0,
            )
        } as usize;
        let addr_buf = &mut addr_buf[..size];
        Ok((
            size,
            CString::new(addr_buf)
                .expect("CString::new")
                .into_string()
                .expect("CString::into_string")
                .parse::<SocketAddr>()
                .expect("String::parse::<SocketAddr>"),
        ))
    }
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize> {
        let addr = addr.to_socket_addrs()?.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No address.",
        ));
        let addr_s = CString::new(addr?.to_string()).expect("CString::new");
        let sent = unsafe {
            sock_send_to(
                self.as_raw_fd(),
                buf.as_ptr() as *const u8,
                buf.len() as u32,
                addr_s.as_ptr() as *const u8,
                addr_s.as_bytes().len() as u32,
                0,
            )
        } as usize;
        Ok(sent)
    }
}
