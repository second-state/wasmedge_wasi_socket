#[link(wasm_import_module = "ssvm")]
extern "C" {
    pub fn ssvm_sock_open(fd: u32, addr_family: u8, sock_type: u8) -> u32;
}
