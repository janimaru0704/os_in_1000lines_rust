#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        core::ptr::write_volatile(0x8020_0000 as *mut u32, 123);
    }
    loop {}
}