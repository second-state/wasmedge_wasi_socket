use std::ffi::CString;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::wasi::prelude::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum AddressFamily {
    Unspec,
    Inet4,
    Inet6,
}

#[allow(unreachable_patterns)]
impl From<&SocketAddr> for AddressFamily {
    fn from(addr: &SocketAddr) -> Self {
        match addr {
            SocketAddr::V4(_) => AddressFamily::Inet4,
            SocketAddr::V6(_) => AddressFamily::Inet6,
            _ => AddressFamily::Unspec,
        }
    }
}

impl AddressFamily {
    pub fn is_unspec(&self) -> bool {
        matches!(*self, AddressFamily::Unspec)
    }

    pub fn is_v4(&self) -> bool {
        matches!(*self, AddressFamily::Inet4)
    }

    pub fn is_v6(&self) -> bool {
        matches!(*self, AddressFamily::Inet6)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketType {
    Any,
    Datagram,
    Stream,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WasiAddress {
    pub buf: *const u8,
    pub size: usize,
}

unsafe impl Send for WasiAddress {}

#[derive(Copy, Clone, Debug)]
#[repr(u16, align(2))]
pub enum AiFlags {
    AiPassive,
    AiCanonname,
    AiNumericHost,
    AiNumericServ,
    AiV4Mapped,
    AiAll,
    AiAddrConfig,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum AiProtocol {
    IPProtoIP,
    IPProtoTCP,
    IPProtoUDP,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct WasiSockaddr {
    pub family: AddressFamily,
    pub sa_data_len: u32,
    pub sa_data: *mut u8,
}

impl WasiSockaddr {
    pub fn new(family: AddressFamily, sa_data: &mut [u8]) -> WasiSockaddr {
        WasiSockaddr {
            family,
            sa_data_len: 14,
            sa_data: sa_data.as_mut_ptr(),
        }
    }
}

impl Default for WasiSockaddr {
    fn default() -> WasiSockaddr {
        WasiSockaddr {
            family: AddressFamily::Inet4,
            sa_data_len: 14,
            sa_data: std::ptr::null_mut(),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed(4))]
pub struct WasiAddrinfo {
    pub ai_flags: AiFlags,
    pub ai_family: AddressFamily,
    pub ai_socktype: SocketType,
    pub ai_protocol: AiProtocol,
    pub ai_addrlen: u32,
    pub ai_addr: *mut WasiSockaddr,
    pub ai_canonname: *mut u8,
    pub ai_canonnamelen: u32,
    pub ai_next: *mut WasiAddrinfo,
}

impl WasiAddrinfo {
    pub fn default() -> WasiAddrinfo {
        WasiAddrinfo {
            ai_flags: AiFlags::AiPassive,
            ai_family: AddressFamily::Inet4,
            ai_socktype: SocketType::Stream,
            ai_protocol: AiProtocol::IPProtoTCP,
            ai_addr: std::ptr::null_mut(),
            ai_addrlen: 0,
            ai_canonname: std::ptr::null_mut(),
            ai_canonnamelen: 0,
            ai_next: std::ptr::null_mut(),
        }
    }

    /// Get Address Information
    ///
    /// As calling FFI, use buffer as parameter in order to avoid memory leak.
    pub fn get_addrinfo(
        node: &str,
        service: &str,
        hints: &WasiAddrinfo,
        max_reslen: usize,
        sockaddr: &mut Vec<WasiSockaddr>,
        sockbuff: &mut Vec<[u8; 26]>,
        ai_canonname: &mut Vec<String>,
    ) -> io::Result<Vec<WasiAddrinfo>> {
        let mut node = node.to_string();
        let mut service = service.to_string();

        if !node.ends_with('\0') {
            node.push('\0');
        }

        if !service.ends_with('\0') {
            service.push('\0');
        }

        let mut res_len: u32 = 0;
        sockbuff.resize(max_reslen, [0u8; 26]);
        ai_canonname.resize(max_reslen, String::with_capacity(30));
        sockaddr.resize(max_reslen, WasiSockaddr::default());
        let mut wasiaddrinfo_array: Vec<WasiAddrinfo> = vec![WasiAddrinfo::default(); max_reslen];

        for i in 0..max_reslen {
            sockaddr[i].sa_data = sockbuff[i].as_mut_ptr();
            wasiaddrinfo_array[i].ai_addr = &mut sockaddr[i];
            wasiaddrinfo_array[i].ai_canonname = ai_canonname[i].as_mut_ptr();
            if i > 0 {
                wasiaddrinfo_array[i - 1].ai_next = &mut wasiaddrinfo_array[i];
            }
        }
        let mut res = wasiaddrinfo_array.as_mut_ptr() as u32;

        unsafe {
            let return_code = sock_getaddrinfo(
                node.as_ptr(),
                node.len() as u32,
                service.as_ptr(),
                service.len() as u32,
                hints as *const WasiAddrinfo,
                &mut res,
                max_reslen as u32,
                &mut res_len,
            );
            match return_code {
                0 => Ok(wasiaddrinfo_array[..res_len as usize].to_vec()),
                _ => Err(std::io::Error::last_os_error()),
            }
        }
    }
}

#[repr(C)]
pub struct IovecRead {
    pub buf: *mut u8,
    pub size: usize,
}

#[repr(C)]
pub struct IovecWrite {
    pub buf: *const u8,
    pub size: usize,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketOptLevel {
    SolSocket = 0,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8, align(1))]
pub enum SocketOptName {
    SoReuseaddr = 0,
    SoType = 1,
    SoError = 2,
    SoDontroute = 3,
    SoBroadcast = 4,
    SoSndbuf = 5,
    SoRcvbuf = 6,
    SoKeepalive = 7,
    SoOobinline = 8,
    SoLinger = 9,
    SoRcvlowat = 10,
    SoRcvtimeo = 11,
    SoSndtimeo = 12,
    SoAcceptconn = 13,
}

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

fn fcntl_add(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous | flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

/// Remove `flag` to the current set flags of `F_GETFD`.
fn fcntl_remove(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous & !flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

mod wasi_sock {
    use super::{IovecRead, IovecWrite, WasiAddress, WasiAddrinfo};

    #[link(wasm_import_module = "wasi_snapshot_preview1")]
    extern "C" {
        pub fn sock_open(addr_family: u8, sock_type: u8, fd: *mut u32) -> u32;
        pub fn sock_bind(fd: u32, addr: *mut WasiAddress, port: u32) -> u32;
        pub fn sock_listen(fd: u32, backlog: u32) -> u32;
        pub fn sock_accept(fd: u32, fd: *mut u32) -> u32;
        pub fn sock_connect(fd: u32, addr: *mut WasiAddress, port: u32) -> u32;
        pub fn sock_recv(
            fd: u32,
            buf: *mut IovecRead,
            buf_len: usize,
            flags: u16,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;
        pub fn sock_recv_from(
            fd: u32,
            buf: *mut u8,
            buf_len: u32,
            addr: *mut u8,
            flags: u16,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;
        pub fn sock_send(
            fd: u32,
            buf: *const IovecWrite,
            buf_len: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;
        pub fn sock_send_to(
            fd: u32,
            buf: *const u8,
            buf_len: u32,
            addr: *const u8,
            port: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;
        pub fn sock_shutdown(fd: u32, flags: u8) -> u32;
        pub fn sock_getaddrinfo(
            node: *const u8,
            node_len: u32,
            server: *const u8,
            server_len: u32,
            hint: *const WasiAddrinfo,
            res: *mut u32,
            max_len: u32,
            res_len: *mut u32,
        ) -> u32;
        pub fn sock_getpeeraddr(
            fd: u32,
            addr: *mut WasiAddress,
            addr_type: *mut u32,
            port: *mut u32,
        ) -> u32;
        pub fn sock_getlocaladdr(
            fd: u32,
            addr: *mut WasiAddress,
            addr_type: *mut u32,
            port: *mut u32,
        ) -> u32;
        pub fn sock_getsockopt(
            fd: u32,
            level: i32,
            name: i32,
            flag: *mut i32,
            flag_size: *mut u32,
        ) -> u32;

        #[allow(unused)]
        pub fn sock_setsockopt(
            fd: u32,
            level: i32,
            name: i32,
            flag: *const i32,
            flag_size: u32,
        ) -> u32;
    }
}

#[derive(Debug)]
pub struct Socket {
    fd: RawFd,
}

use wasi_sock::*;
impl Socket {
    pub fn new(addr_family: AddressFamily, sock_kind: SocketType) -> io::Result<Self> {
        unsafe {
            let mut fd = 0;
            let res = sock_open(addr_family as u8, sock_kind as u8, &mut fd);
            if res == 0 {
                Ok(Socket { fd: fd as i32 })
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let mut send_len: u32 = 0;
            let vec = IovecWrite {
                buf: buf.as_ptr(),
                size: buf.len(),
            };
            let res = sock_send(self.as_raw_fd() as u32, &vec, 1, 0, &mut send_len);
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let addr_s = CString::new(addr.to_string()).expect("CString::new");
        let port = 0;
        let flags = 0;
        let mut send_len: u32 = 0;
        unsafe {
            let res = sock_send_to(
                self.fd as u32,
                buf.as_ptr() as *const u8,
                buf.len() as u32,
                addr_s.as_ptr() as *const u8,
                port,
                flags,
                &mut send_len
                );
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let flags = 0;
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut vec = IovecRead {
            buf: buf.as_mut_ptr(),
            size: buf.len(),
        };

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                &mut vec,
                1,
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok(recv_len)
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let flags = 0;
        let mut addr_buf = [0; 32];
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                buf.as_ptr() as *mut u8,
                buf.len() as u32,
                addr_buf.as_ptr() as *mut u8,
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let addr_buf = &mut addr_buf[..recv_len];
                Ok((
                        recv_len,
                        CString::new(addr_buf)
                        .expect("CString::new")
                        .into_string()
                        .expect("CString::into_string")
                        .parse::<SocketAddr>()
                        .expect("String::parse::<SocketAddr>"),
                        ))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let fd = self.as_raw_fd();
        if nonblocking {
            fcntl_add(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        } else {
            fcntl_remove(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        }
    }

    pub fn connect(&self, addrs: &SocketAddr) -> io::Result<()> {
        let fd = self.as_raw_fd();
        let vaddr;
        let port;
        if let SocketAddr::V4(addrs) = addrs {
            vaddr = addrs.ip().octets();
            port = addrs.port();
        } else {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }
        let mut addr = WasiAddress {
            buf: vaddr.as_ptr(),
            size: 4,
        };

        unsafe {
            let res = sock_connect(fd as u32, &mut addr, port as u32);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(())
            }
        }
    }

    pub fn bind(&self, addrs: &SocketAddr) -> io::Result<()> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut vaddr: [u8; 16] = [0; 16];
            let port;
            let size;
            match addrs {
                SocketAddr::V4(addr) => {
                    let ip = addr.ip().octets();
                    (&mut vaddr[0..4]).clone_from_slice(&ip);
                    port = addr.port();
                    size = 4;
                }
                SocketAddr::V6(addr) => {
                    let ip = addr.ip().octets();
                    vaddr.clone_from_slice(&ip);
                    port = addr.port();
                    size = 16;
                }
            }
            let mut addr = WasiAddress {
                buf: vaddr.as_ptr(),
                size,
            };
            let res = sock_bind(fd as u32, &mut addr, port as u32);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(())
            }
        }
    }

    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        unsafe {
            let fd = self.as_raw_fd();
            let res = sock_listen(fd as u32, backlog as u32);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                Ok(())
            }
        }
    }

    pub fn accept(&self, nonblocking: bool) -> io::Result<Self> {
        unsafe {
            let mut fd: u32 = 0;
            let res = sock_accept(self.as_raw_fd() as u32, &mut fd);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                let s = Socket { fd: fd as i32 };
                s.set_nonblocking(nonblocking)?;
                Ok(s)
            }
        }
    }

    pub fn get_local(&self) -> io::Result<SocketAddr> {
        unsafe {
            let fd = self.fd;
            let addr_buf = [0u8; 16];
            let mut addr = WasiAddress {
                buf: addr_buf.as_ptr(),
                size: 16,
            };
            let mut addr_type = 0;
            let mut port = 0;
            let res = sock_getlocaladdr(fd as u32, &mut addr, &mut addr_type, &mut port);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                if addr_type == 4 {
                    let ip_addr = Ipv4Addr::new(addr_buf[0], addr_buf[1], addr_buf[2], addr_buf[3]);
                    Ok(SocketAddr::V4(SocketAddrV4::new(ip_addr, port as u16)))
                } else if addr_type == 6 {
                    let ip_addr = Ipv6Addr::from(addr_buf);
                    Ok(SocketAddr::V6(SocketAddrV6::new(
                        ip_addr,
                        port as u16,
                        0,
                        0,
                    )))
                } else {
                    Err(io::Error::from(io::ErrorKind::Unsupported))
                }
            }
        }
    }

    pub fn get_peer(&self) -> io::Result<SocketAddr> {
        unsafe {
            let fd = self.fd;
            let addr_buf = [0u8; 16];
            let mut addr = WasiAddress {
                buf: addr_buf.as_ptr(),
                size: 16,
            };
            let mut addr_type = 0;
            let mut port = 0;
            let res = sock_getpeeraddr(fd as u32, &mut addr, &mut addr_type, &mut port);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res as i32))
            } else {
                if addr_type == 4 {
                    let ip_addr = Ipv4Addr::new(addr_buf[0], addr_buf[1], addr_buf[2], addr_buf[3]);
                    Ok(SocketAddr::V4(SocketAddrV4::new(ip_addr, port as u16)))
                } else if addr_type == 6 {
                    let ip_addr = Ipv6Addr::from(addr_buf);
                    Ok(SocketAddr::V6(SocketAddrV6::new(
                        ip_addr,
                        port as u16,
                        0,
                        0,
                    )))
                } else {
                    Err(io::Error::from(io::ErrorKind::Unsupported))
                }
            }
        }
    }

    pub fn take_error(&self) -> io::Result<()> {
        unsafe {
            let fd = self.fd;
            let mut error = 0;
            let mut len = std::mem::size_of::<i32>() as u32;
            let res = sock_getsockopt(
                fd as u32,
                SocketOptLevel::SolSocket as i32,
                SocketOptName::SoError as i32,
                &mut error,
                &mut len,
            );
            if res == 0 && error == 0 {
                Ok(())
            } else if res == 0 && error != 0 {
                Err(io::Error::from_raw_os_error(error))
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }

    pub fn setsockopt<T>(
        &self,
        level: SocketOptLevel,
        name: SocketOptName,
        payload: T,
    ) -> io::Result<()> {
        unsafe {
            let fd = self.fd as u32;
            let flag = &payload as *const T as *const i32;
            let flag_size = std::mem::size_of::<T>() as u32;
            let e = sock_setsockopt(fd, level as u8 as i32, name as u8 as i32, flag, flag_size);
            if e == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(e as i32))
            }
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        unsafe {
            let flags = match how {
                Shutdown::Read => 1,
                Shutdown::Write => 2,
                Shutdown::Both => 3,
            };
            let res = sock_shutdown(self.as_raw_fd() as u32, flags);
            if res == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(res as i32))
            }
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.shutdown(Shutdown::Both);
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Socket { fd }
    }
}
