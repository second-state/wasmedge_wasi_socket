use std::io::{Read, Result, Write};
pub use std::net::{Shutdown, SocketAddr, ToSocketAddrs};

#[link(wasm_import_module = "ssvm")]
extern "C" {
    pub fn ssvm_sock_open(addr_family: u8, sock_type: u8) -> u32;
    pub fn ssvm_sock_close(fd: u32);
    pub fn ssvm_sock_bind(fd: u32, addr: *const u8, addr_len: u32);
    pub fn ssvm_sock_connect(fd: u32, addr: *const u8, addr_len: u32);
    pub fn ssvm_sock_recv(fd: u32, buf: *mut u8, buf_len: u32, flags: u16) -> u32;
    pub fn ssvm_sock_recv_from(
        fd: u32,
        buf: *mut u8,
        buf_len: u32,
        addr: *const u8,
        addr_len: u32,
        flags: u16,
    ) -> u32;
    pub fn ssvm_sock_send(fd: u32, buf: *const u8, buf_len: u32, flags: u16) -> u32;
    pub fn ssvm_sock_send_to(
        fd: u32,
        buf: *const u8,
        buf_len: u32,
        addr: *const u8,
        addr_len: u32,
        flags: u16,
    ) -> u32;
    pub fn ssvm_sock_shutdown(fd: u32, flags: u8);
}

#[non_exhaustive]
pub struct TcpStream;

#[non_exhaustive]
pub struct TcpListener;

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<TcpStream> {
        todo!();
    }
    pub fn shutdown(&self, how: Shutdown) -> Result<()> {
        todo!();
    }
}

impl Read for TcpStream {
    fn read(&mut self, _: &mut [u8]) -> Result<usize> {
        todo!()
    }
}

impl Write for TcpStream {
    fn write(&mut self, _: &[u8]) -> Result<usize> {
        todo!()
    }
    fn flush(&mut self) -> Result<()> {
        todo!()
    }
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<TcpListener> {
        todo!();
    }
    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        todo!();
    }
}
