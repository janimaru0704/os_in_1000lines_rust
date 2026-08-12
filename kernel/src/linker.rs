unsafe extern "C" {
    pub static mut __bss: u8;
    pub static mut __bss_end: u8;
    pub static mut __stack_top: u8;
    pub static mut __free_ram: u8;
    pub static mut __free_ram_end: u8;
    pub static mut __kernel_base: u8;
}
