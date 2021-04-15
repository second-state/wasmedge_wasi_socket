#[link(wasm_import_module = "ssvm")]
extern "C" {
    pub fn ssvm_sock_open(addr_family: u8, sock_type: u8) -> u32;
    pub fn ssvm_sock_close(fd: u32);
    pub fn ssvm_sock_bind(fd: u32, addr: *const u8, addr_len: u32);
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
